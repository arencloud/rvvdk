#[path = "support/chain.rs"]
#[allow(dead_code)]
mod f;
use f::{Graph, node, text};
use rvvdk_core::{BlockDevice, EndpointIdentity};
use rvvdk_vmdk::{
    BackingError, Descriptor, Limits, ParentResolver, SparseChain, SparseChainError as E,
    SparseChainLimits, SparseChainSource, SparseDescriptor, SparseLayerDescriptor,
};
use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};
fn load(g: &Graph) -> Result<SparseChain, E> {
    SparseChain::load(g.root(), g, SparseChainLimits::default())
}
#[test]
fn parent_syntax_is_opt_in_and_bounded() {
    let t = text(7, Some((8, "../parent.vmdk")), false);
    let d = SparseLayerDescriptor::parse(t.as_bytes()).unwrap();
    assert_eq!(d.cid(), 7);
    assert_eq!(d.parent().unwrap().cid, 8);
    assert_eq!(d.parent().unwrap().file_name_hint, "../parent.vmdk");
    assert!(SparseDescriptor::parse(t.as_bytes()).is_err());
    assert!(Descriptor::parse(t.as_bytes()).is_err());
    let mut padded = t.as_bytes().to_vec();
    padded.extend([0; 64]);
    assert!(SparseLayerDescriptor::parse(&padded).is_ok());
    assert!(
        SparseLayerDescriptor::parse_with_limits(
            &padded,
            Limits {
                descriptor_bytes: padded.len() - 1,
                ..Limits::default()
            }
        )
        .is_err()
    );
    assert!(
        SparseLayerDescriptor::parse_with_limits(
            t.as_bytes(),
            Limits {
                filename_bytes: 5,
                ..Limits::default()
            }
        )
        .is_err()
    );
    for bad in [
        t.replace("parentFileNameHint=\"../parent.vmdk\"\n", ""),
        t.replace("../parent.vmdk", ""),
        t.replace("parentCID=00000008", "parentCID=ffffffff"),
        t.replace("parentCID=00000008", "parentCID=xyz"),
        t.replace(
            "parentFileNameHint=",
            "parentFileNameHint=\"x\"\nparentFileNameHint=",
        ),
        t.replace("\"../parent.vmdk\"", "parent.vmdk"),
    ] {
        assert!(SparseLayerDescriptor::parse(bad.as_bytes()).is_err());
    }
}
#[test]
fn retains_external_and_embedded_leaf_to_base_maps_and_exact_read_counters() {
    for embedded in [false, true] {
        let g = Graph::new(3, embedded, true);
        let chain = load(&g).unwrap();
        assert_eq!(g.calls.load(Ordering::Relaxed), 2);
        assert_eq!(
            chain.layers().iter().map(|l| l.cid()).collect::<Vec<_>>(),
            [1, 2, 3]
        );
        assert_eq!(chain.layers()[2].parent_cid(), None);
        for layer in chain.layers() {
            assert_eq!(layer.size_bytes(), 1 << 20);
            assert_eq!(layer.metadata()[0].grain_sectors()[2], 0);
        }
        let actual: u64 = g
            .nodes
            .iter()
            .map(|n| {
                n.backing.bytes.load(Ordering::Relaxed)
                    + if embedded {
                        0
                    } else {
                        n.descriptor.bytes.load(Ordering::Relaxed)
                    }
            })
            .sum();
        assert_eq!(actual, chain.metadata_read_bytes());
        println!(
            "R56_PROFILE embedded={embedded} depth=3 memory={} reads={} descriptors={}",
            chain.reserved_memory_bytes(),
            chain.metadata_read_bytes(),
            chain.descriptor_bytes()
        );
        drop(g);
        chain.revalidate().unwrap();
    }
}
#[test]
fn terminal_base_never_calls_parent_resolver_and_equal_cids_are_not_cycles() {
    let mut g = Graph::new(2, false, false);
    g.nodes[1] = node(&text(1, None, false), false, false);
    g.nodes[0] = node(&text(1, Some((1, "parent.vmdk")), false), false, false);
    assert_eq!(load(&g).unwrap().layers().len(), 2);
    let g = Graph::new(1, false, false);
    assert_eq!(load(&g).unwrap().layers().len(), 1);
    assert_eq!(g.calls.load(Ordering::Relaxed), 0);
}
#[test]
fn missing_and_denied_parents_are_distinct() {
    let mut g = Graph::new(2, false, false);
    g.nodes.pop();
    assert!(matches!(load(&g), Err(E::MissingParent)));
    g.nodes[0] = node(&text(1, Some((2, "../private.vmdk")), false), false, false);
    assert!(matches!(
        load(&g),
        Err(E::Resolve(BackingError::UnsafeReference))
    ));
}
#[test]
fn cid_and_capacity_mismatch_fail_before_parent_backing_reads() {
    for capacity in [false, true] {
        let mut g = Graph::new(2, false, true);
        let t = if capacity {
            text(2, None, false).replace("RW 2048", "RW 4096")
        } else {
            text(3, None, false)
        };
        g.nodes[1] = node(&t, false, true);
        let e = load(&g).err().unwrap();
        assert!(if capacity {
            matches!(e, E::CapacityMismatch)
        } else {
            matches!(e, E::CidMismatch)
        });
        assert_eq!(g.nodes[1].backing.calls.load(Ordering::Relaxed), 0);
    }
}
struct Loop {
    source: SparseChainSource,
    calls: AtomicU64,
}
impl ParentResolver for Loop {
    fn resolve_parent(
        &self,
        _: EndpointIdentity,
        _: &str,
    ) -> Result<Option<SparseChainSource>, BackingError> {
        self.calls.fetch_add(1, Ordering::Relaxed);
        Ok(Some(self.source.clone()))
    }
}
#[test]
fn self_and_long_cycles_reject_by_object_identity_before_rereading() {
    let g = Graph::new(2, false, true);
    let looping = Loop {
        source: g.root(),
        calls: AtomicU64::new(0),
    };
    assert!(matches!(
        SparseChain::load(g.root(), &looping, SparseChainLimits::default()),
        Err(E::RepeatedIdentity)
    ));
    assert_eq!(g.nodes[0].descriptor.calls.load(Ordering::Relaxed), 1);
    struct Cycle<'a>(&'a Graph);
    impl ParentResolver for Cycle<'_> {
        fn resolve_parent(
            &self,
            c: EndpointIdentity,
            h: &str,
        ) -> Result<Option<SparseChainSource>, BackingError> {
            Ok(Some(
                self.0
                    .resolve_parent(c, h)?
                    .unwrap_or_else(|| self.0.root()),
            ))
        }
    }
    let mut g = Graph::new(2, false, true);
    g.nodes[1] = node(&text(2, Some((1, "parent.vmdk")), false), false, true);
    assert!(matches!(
        SparseChain::load(g.root(), &Cycle(&g), SparseChainLimits::default()),
        Err(E::RepeatedIdentity)
    ));
    assert_eq!(g.nodes[0].descriptor.calls.load(Ordering::Relaxed), 1);
}
#[test]
fn layer_boundary_checks_before_following_next_hint() {
    let g = Graph::new(3, false, true);
    assert!(matches!(
        SparseChain::load(
            g.root(),
            &g,
            SparseChainLimits {
                layers: 0,
                ..Default::default()
            }
        ),
        Err(E::Limit("layers"))
    ));
    assert_eq!(g.nodes[0].descriptor.calls.load(Ordering::Relaxed), 0);
    assert!(matches!(
        SparseChain::load(
            g.root(),
            &g,
            SparseChainLimits {
                layers: 2,
                ..Default::default()
            }
        ),
        Err(E::Limit("layers"))
    ));
    assert_eq!(g.calls.load(Ordering::Relaxed), 1);
    assert_eq!(g.nodes[2].descriptor.calls.load(Ordering::Relaxed), 0);
    let g = Graph::new(2, false, false);
    assert!(
        SparseChain::load(
            g.root(),
            &g,
            SparseChainLimits {
                layers: 2,
                ..Default::default()
            }
        )
        .is_ok()
    );
}
#[test]
fn aggregate_budgets_have_exact_success_and_one_byte_failure_boundaries() {
    let g = Graph::new(3, true, true);
    let c = load(&g).unwrap();
    let limits = SparseChainLimits {
        memory_bytes: c.reserved_memory_bytes(),
        read_bytes: c.metadata_read_bytes(),
        descriptor_bytes: c.descriptor_bytes(),
        extents: 3,
        ..Default::default()
    };
    assert!(SparseChain::load(g.root(), &g, limits).is_ok());
    for limits in [
        SparseChainLimits {
            memory_bytes: limits.memory_bytes - 1,
            ..limits
        },
        SparseChainLimits {
            read_bytes: limits.read_bytes - 1,
            ..limits
        },
        SparseChainLimits {
            descriptor_bytes: limits.descriptor_bytes - 1,
            ..limits
        },
        SparseChainLimits {
            extents: 2,
            ..limits
        },
    ] {
        assert!(SparseChain::load(g.root(), &g, limits).is_err());
    }
    let g = Graph::new(1, false, true);
    let len = g.nodes[0].descriptor.size();
    assert!(
        SparseChain::load(
            g.root(),
            &g,
            SparseChainLimits {
                descriptor_bytes: len - 1,
                ..Default::default()
            }
        )
        .is_err()
    );
    assert_eq!(g.nodes[0].descriptor.calls.load(Ordering::Relaxed), 0);
    assert_eq!(g.nodes[0].backing.calls.load(Ordering::Relaxed), 0);
    assert!(
        SparseChain::load(
            g.root(),
            &g,
            SparseChainLimits {
                layers: usize::MAX,
                ..Default::default()
            }
        )
        .is_err()
    );
}
#[test]
fn unknown_descriptor_or_backing_identity_rejects_before_that_source_read() {
    for backing in [false, true] {
        let g = Graph::new(1, false, true);
        let target = if backing {
            &g.nodes[0].backing
        } else {
            &g.nodes[0].descriptor
        };
        target.unknown.store(true, Ordering::Relaxed);
        assert!(matches!(load(&g), Err(E::Identity)));
        assert_eq!(target.calls.load(Ordering::Relaxed), 0);
    }
}
#[test]
fn rejects_shared_backings_and_parent_descriptor_aliases() {
    let mut g = Graph::new(2, false, true);
    g.nodes[1].source.backings = g.nodes[0].source.backings.clone();
    assert!(matches!(load(&g), Err(E::RepeatedIdentity)));
    let g = Graph::new(2, false, true);
    let looping = Loop {
        source: SparseChainSource {
            descriptor: g.nodes[0].backing.clone(),
            ..g.nodes[1].source.clone()
        },
        calls: AtomicU64::new(0),
    };
    assert!(matches!(
        SparseChain::load(g.root(), &looping, SparseChainLimits::default()),
        Err(E::RepeatedIdentity)
    ));
}
#[test]
fn embedded_entry_is_bound_to_same_container_and_parent_fields() {
    let mut g = Graph::new(1, true, true);
    let another = node(&text(1, None, false), true, true);
    g.nodes[0].source.backings = another.source.backings;
    assert!(matches!(load(&g), Err(E::Identity)));
    assert_eq!(another.backing.calls.load(Ordering::Relaxed), 0);
    let mut g = Graph::new(2, false, true);
    let another = node(&text(1, Some((9, "parent.vmdk")), false), false, true);
    g.nodes[0].source.backings = another.source.backings;
    assert!(matches!(load(&g), Err(E::Metadata(_))));
    let mut g = Graph::new(1, true, true);
    g.nodes[0] = node(&text(1, None, true), true, true);
    assert!(matches!(load(&g), Err(E::Invalid(_))));
}
#[test]
fn multiple_split_extents_and_different_parent_geometry_are_allowed() {
    struct Split(Vec<Arc<f::Tracked>>);
    impl rvvdk_vmdk::BackingResolver for Split {
        fn resolve(&self, n: &str) -> Result<Arc<dyn BlockDevice>, BackingError> {
            Ok(self.0[usize::from(n == "second.vmdk")].clone())
        }
    }
    let mut g = Graph::new(2, false, true);
    // Two separate 1 MiB extents in each layer; backing namespace is per layer.
    for i in 0..2 {
        let t = text(
            i as u32 + 1,
            if i == 0 {
                Some((2, "parent.vmdk"))
            } else {
                None
            },
            true,
        ) + "RW 2048 SPARSE \"second.vmdk\"\n";
        let a = node(&t, false, true);
        let b = node(&t, false, true);
        g.nodes[i] = a;
        g.nodes[i].source.backings = Arc::new(Split(vec![g.nodes[i].backing.clone(), b.backing]));
    }
    let parent_text = text(2, None, false).replace("RW 2048", "RW 4096");
    g.nodes[1] = node(&parent_text, false, true);
    g.nodes[1]
        .backing
        .device
        .write_all_at(12, &4096u64.to_le_bytes())
        .unwrap();
    let c = load(&g).unwrap();
    assert_eq!(c.layers()[0].size_bytes(), 2 << 20);
    assert_eq!(c.layers()[0].metadata().len(), 2);
    assert_eq!(c.layers()[1].metadata().len(), 1);
}
#[test]
fn revalidation_and_final_observation_detect_prior_source_changes() {
    let g = Graph::new(2, false, false);
    let c = load(&g).unwrap();
    g.nodes[0].descriptor.changed.store(true, Ordering::Relaxed);
    assert!(c.revalidate().is_err());
    g.nodes[0]
        .descriptor
        .changed
        .store(false, Ordering::Relaxed);
    g.nodes[1].backing.unknown.store(true, Ordering::Relaxed);
    assert!(c.revalidate().is_err());
    struct Changing<'a>(&'a Graph);
    impl ParentResolver for Changing<'_> {
        fn resolve_parent(
            &self,
            c: EndpointIdentity,
            h: &str,
        ) -> Result<Option<SparseChainSource>, BackingError> {
            self.0.nodes[0]
                .descriptor
                .changed
                .store(true, Ordering::Relaxed);
            self.0.resolve_parent(c, h)
        }
    }
    let g = Graph::new(2, false, false);
    assert!(SparseChain::load(g.root(), &Changing(&g), SparseChainLimits::default()).is_err());
}
#[test]
fn descriptor_and_metadata_io_errors_propagate_without_parent_fallback() {
    for backing in [false, true] {
        let g = Graph::new(2, false, true);
        let target = if backing {
            &g.nodes[0].backing
        } else {
            &g.nodes[0].descriptor
        };
        target.fail.store(true, Ordering::Relaxed);
        assert!(load(&g).is_err());
        assert_eq!(g.calls.load(Ordering::Relaxed), 0);
    }
}

#[test]
fn duplicate_extent_handles_and_per_layer_limits_are_rejected() {
    let mut g = Graph::new(1, false, true);
    let t = text(1, None, true) + "RW 2048 SPARSE \"second.vmdk\"\n";
    g.nodes[0] = node(&t, false, true);
    assert!(matches!(load(&g), Err(E::RepeatedIdentity)));
    let g = Graph::new(1, true, true);
    let mut limits = SparseChainLimits::default();
    limits.metadata.descriptor.descriptor_bytes = 10239;
    assert!(SparseChain::load(g.root(), &g, limits).is_err());
    assert_eq!(g.nodes[0].descriptor.calls.load(Ordering::Relaxed), 1);
    assert_eq!(g.nodes[0].descriptor.bytes.load(Ordering::Relaxed), 512);
}

#[test]
fn default_depth_accepts_sixteen_and_rejects_seventeen_without_opening_last() {
    let g = Graph::new(16, true, false);
    assert_eq!(load(&g).unwrap().layers().len(), 16);
    let g = Graph::new(17, false, true);
    assert!(matches!(load(&g), Err(E::Limit("layers"))));
    assert_eq!(g.calls.load(Ordering::Relaxed), 15);
    assert_eq!(g.nodes[16].descriptor.calls.load(Ordering::Relaxed), 0);
}
