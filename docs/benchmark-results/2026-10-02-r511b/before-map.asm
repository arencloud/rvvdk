
target/r511b/reference/map-before:     file format elf64-x86-64


Disassembly of section .text:

00000000000b6320 <rvvdk_vmdk::stream_map::StreamMap::read_from>:
   b6320:	push   %rbp
   b6321:	push   %r15
   b6323:	push   %r14
   b6325:	push   %r13
   b6327:	push   %r12
   b6329:	push   %rbx
   b632a:	sub    $0x1000,%rsp
   b6331:	movq   $0x0,(%rsp)
   b6339:	sub    $0x5b8,%rsp
   b6340:	mov    %r8,%rbp
   b6343:	mov    %rdi,%r15
   b6346:	mov    %rsi,0x240(%rsp)
   b634e:	mov    %rdx,0x248(%rsp)
   b6356:	mov    %rcx,0x88(%rsp)
   b635e:	movups (%r8),%xmm0
   b6362:	movups 0x10(%r8),%xmm1
   b6367:	movups 0x20(%r8),%xmm2
   b636c:	movups 0x30(%r8),%xmm3
   b6371:	movaps %xmm2,0x570(%rsp)
   b6379:	movaps %xmm0,0x550(%rsp)
   b6381:	movaps %xmm1,0x560(%rsp)
   b6389:	movaps %xmm3,0x580(%rsp)
   b6391:	movups 0x40(%r8),%xmm0
   b6396:	movaps %xmm0,0x590(%rsp)
   b639e:	mov    0x50(%r8),%rax
   b63a2:	mov    %rax,0x5a0(%rsp)
   b63aa:	mov    0x578(%rsp),%rax
   b63b2:	mov    0x60(%r8),%rbx
   b63b6:	cmp    %rax,%rbx
   b63b9:	cmovb  %rbx,%rax
   b63bd:	mov    %rax,0x578(%rsp)
   b63c5:	lea    0x5b0(%rsp),%rdi
   b63cd:	lea    0x240(%rsp),%r13
   b63d5:	lea    0x550(%rsp),%rax
   b63dd:	mov    %r13,%rsi
   b63e0:	mov    %rcx,%rdx
   b63e3:	mov    %rax,%rcx
   b63e6:	call   b0ed0 <rvvdk_vmdk::stream::StreamEnvelope::read_from>
   b63eb:	mov    0x5b0(%rsp),%r14
   b63f3:	movups 0x5b8(%rsp),%xmm0
   b63fb:	movaps %xmm0,0xdb0(%rsp)
   b6403:	movups 0x5c8(%rsp),%xmm0
   b640b:	movaps %xmm0,0xdc0(%rsp)
   b6413:	cmp    $0x3,%r14
   b6417:	jne    b643f <rvvdk_vmdk::stream_map::StreamMap::read_from+0x11f>
   b6419:	movaps 0xdb0(%rsp),%xmm0
   b6421:	movaps 0xdc0(%rsp),%xmm1
   b6429:	movups %xmm1,0x18(%r15)
   b642e:	movups %xmm0,0x8(%r15)
   b6433:	movq   $0x3,(%r15)
   b643a:	jmp    b6b85 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x865>
   b643f:	mov    0x608(%rsp),%rax
   b6447:	mov    %rax,0x1b0(%rsp)
   b644f:	movups 0x5d8(%rsp),%xmm0
   b6457:	movups 0x5e8(%rsp),%xmm1
   b645f:	movups 0x5f8(%rsp),%xmm2
   b6467:	movaps %xmm2,0x1a0(%rsp)
   b646f:	movaps %xmm1,0x190(%rsp)
   b6477:	movaps %xmm0,0x180(%rsp)
   b647f:	mov    0x620(%rsp),%r12
   b6487:	mov    0x628(%rsp),%rax
   b648f:	mov    0x628(%rsp),%edx
   b6496:	movups 0x610(%rsp),%xmm0
   b649e:	mov    0x610(%rsp),%rsi
   b64a6:	movaps 0xdb0(%rsp),%xmm1
   b64ae:	movaps 0xdc0(%rsp),%xmm2
   b64b6:	movaps %xmm1,0x330(%rsp)
   b64be:	movaps %xmm2,0x340(%rsp)
   b64c6:	mov    %r14,0x250(%rsp)
   b64ce:	movaps 0xdb0(%rsp),%xmm1
   b64d6:	movaps 0xdc0(%rsp),%xmm2
   b64de:	movups %xmm2,0x268(%rsp)
   b64e6:	movups %xmm1,0x258(%rsp)
   b64ee:	mov    0x1b0(%rsp),%rcx
   b64f6:	mov    %rcx,0x2a8(%rsp)
   b64fe:	movaps 0x180(%rsp),%xmm1
   b6506:	movaps 0x190(%rsp),%xmm2
   b650e:	movaps 0x1a0(%rsp),%xmm3
   b6516:	movups %xmm3,0x298(%rsp)
   b651e:	movups %xmm2,0x288(%rsp)
   b6526:	movups %xmm1,0x278(%rsp)
   b652e:	movups %xmm0,0x2b0(%rsp)
   b6536:	mov    %r12,0x2c0(%rsp)
   b653e:	mov    %rax,0x2c8(%rsp)
   b6546:	mov    0x278(%rsp),%rcx
   b654e:	movabs $0x1000000010000,%rax
   b6558:	cmp    %rax,%rcx
   b655b:	jb     b6584 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x264>
   b655d:	movq   $0xb,0x8(%r15)
   b6565:	lea    -0x95efb(%rip),%rax        # 20671 <anon.321e7b27d4937aed77e095c38e205854.42.llvm.3789170322229662815+0x88>
   b656c:	mov    %rax,0x10(%r15)
   b6570:	movq   $0xe,0x18(%r15)
   b6578:	movq   $0x3,(%r15)
   b657f:	jmp    b6b85 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x865>
   b6584:	mov    %rsi,0x80(%rsp)
   b658c:	mov    %edx,0xe4(%rsp)
   b6593:	mov    %rcx,%rdx
   b6596:	shr    $0x19,%rdx
   b659a:	mov    %rcx,0xd0(%rsp)
   b65a2:	mov    %ecx,%eax
   b65a4:	and    $0x1ff0000,%eax
   b65a9:	cmp    $0x1,%rax
   b65ad:	sbb    $0xffffffffffffffff,%rdx
   b65b1:	mov    %rdx,0x78(%rsp)
   b65b6:	mov    0x258(%rsp),%rax
   b65be:	mov    %rax,0x8(%rsp)
   b65c3:	mov    0x260(%rsp),%rax
   b65cb:	mov    %rax,0x30(%rsp)
   b65d0:	xor    %eax,%eax
   b65d2:	cmp    $0x2,%r14
   b65d6:	cmovne %r14,%rax
   b65da:	xor    %edx,%edx
   b65dc:	mov    %rax,0x10(%rsp)
   b65e1:	cmp    $0x1,%rax
   b65e5:	sete   %dl
   b65e8:	inc    %rdx
   b65eb:	mov    0x2a0(%rsp),%rsi
   b65f3:	lea    0x5b0(%rsp),%rdi
   b65fb:	mov    %rsi,(%rsp)
   b65ff:	mov    %rdx,0x20(%rsp)
   b6604:	call   *0x2106be(%rip)        # 2c6cc8 <_DYNAMIC+0x618>
   b660a:	mov    0x5b0(%rsp),%rax
   b6612:	mov    0x5b8(%rsp),%rcx
   b661a:	cmp    $0x10,%rax
   b661e:	jne    b6650 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x330>
   b6620:	mov    0x58(%rbp),%rax
   b6624:	cmp    %rax,%rcx
   b6627:	jbe    b6671 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x351>
   b6629:	movq   $0xb,0x8(%r15)
   b6631:	lea    -0x9f338(%rip),%rax        # 17300 <anon.d625489d584c397ac22d75864c33158f.35.llvm.5936746164385555759+0x1a0>
   b6638:	mov    %rax,0x10(%r15)
   b663c:	movq   $0x10,0x18(%r15)
   b6644:	movq   $0x3,(%r15)
   b664b:	jmp    b6b85 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x865>
   b6650:	movups 0x5c0(%rsp),%xmm0
   b6658:	movups %xmm0,0x18(%r15)
   b665d:	mov    %rax,0x8(%r15)
   b6661:	mov    %rcx,0x10(%r15)
   b6665:	movq   $0x3,(%r15)
   b666c:	jmp    b6b85 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x865>
   b6671:	mov    %rcx,0x58(%rsp)
   b6676:	mov    %rax,0xa0(%rsp)
   b667e:	mov    %r14,0x98(%rsp)
   b6686:	mov    $0x1,%eax
   b668b:	mov    %rax,0x40(%rsp)
   b6690:	mov    %rax,0x18(%rsp)
   b6695:	mov    (%rsp),%r14
   b6699:	test   %r14,%r14
   b669c:	jne    b687d <rvvdk_vmdk::stream_map::StreamMap::read_from+0x55d>
   b66a2:	xor    %eax,%eax
   b66a4:	cmpl   $0x1,0x10(%rsp)
   b66a9:	mov    $0x0,%ecx
   b66ae:	cmove  %r14,%rcx
   b66b2:	test   %rcx,%rcx
   b66b5:	mov    %rcx,0x60(%rsp)
   b66ba:	jne    b68c6 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x5a6>
   b66c0:	mov    %rax,0x50(%rsp)
   b66c5:	mov    %r13,0x100(%rsp)
   b66cd:	mov    0x88(%rsp),%rax
   b66d5:	mov    %rax,0x108(%rsp)
   b66dd:	mov    %r12,0x110(%rsp)
   b66e5:	mov    %rbx,0x118(%rsp)
   b66ed:	lea    0x5b0(%rsp),%rdi
   b66f5:	mov    %r14,%rsi
   b66f8:	mov    0x20(%rsp),%rdx
   b66fd:	mov    %r14,%r13
   b6700:	mov    0x18(%rsp),%r14
   b6705:	call   *0x2105bd(%rip)        # 2c6cc8 <_DYNAMIC+0x618>
   b670b:	mov    0x5b0(%rsp),%rax
   b6713:	mov    0x5b8(%rsp),%rsi
   b671b:	cmp    $0x10,%rax
   b671f:	jne    b6b39 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x819>
   b6725:	lea    0xdb0(%rsp),%rdi
   b672d:	mov    $0x200,%edx
   b6732:	call   *0x210588(%rip)        # 2c6cc0 <_DYNAMIC+0x610>
   b6738:	mov    0xdb0(%rsp),%rax
   b6740:	mov    0xdb8(%rsp),%rdx
   b6748:	cmp    $0x10,%rax
   b674c:	jne    b679a <rvvdk_vmdk::stream_map::StreamMap::read_from+0x47a>
   b674e:	lea    0x350(%rsp),%rdi
   b6756:	mov    %r12,%rsi
   b6759:	call   *0x210561(%rip)        # 2c6cc0 <_DYNAMIC+0x610>
   b675f:	mov    0x350(%rsp),%rcx
   b6767:	mov    0x358(%rsp),%rax
   b676f:	cmp    $0x10,%rcx
   b6773:	jne    b67b4 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x494>
   b6775:	cmp    %rbx,%rax
   b6778:	jbe    b67ce <rvvdk_vmdk::stream_map::StreamMap::read_from+0x4ae>
   b677a:	movq   $0xb,0x8(%r15)
   b6782:	lea    -0x96126(%rip),%rax        # 20663 <anon.321e7b27d4937aed77e095c38e205854.42.llvm.3789170322229662815+0x7a>
   b6789:	mov    %rax,0x10(%r15)
   b678d:	movq   $0xe,0x18(%r15)
   b6795:	jmp    b6b4e <rvvdk_vmdk::stream_map::StreamMap::read_from+0x82e>
   b679a:	movups 0xdc0(%rsp),%xmm0
   b67a2:	movups %xmm0,0x18(%r15)
   b67a7:	mov    %rax,0x8(%r15)
   b67ab:	mov    %rdx,0x10(%r15)
   b67af:	jmp    b6b4e <rvvdk_vmdk::stream_map::StreamMap::read_from+0x82e>
   b67b4:	movups 0x360(%rsp),%xmm0
   b67bc:	movups %xmm0,0x18(%r15)
   b67c1:	mov    %rcx,0x8(%r15)
   b67c5:	mov    %rax,0x10(%r15)
   b67c9:	jmp    b6b4e <rvvdk_vmdk::stream_map::StreamMap::read_from+0x82e>
   b67ce:	lea    0x5b0(%rsp),%rdi
   b67d6:	lea    0x100(%rsp),%rsi
   b67de:	mov    0x80(%rsp),%rdx
   b67e6:	mov    %r14,%rcx
   b67e9:	mov    %r13,%r8
   b67ec:	call   b61c0 <rvvdk_vmdk::stream_map::Budget<S>::read>
   b67f1:	cmpl   $0x10,0x5b0(%rsp)
   b67f9:	jne    b6918 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x5f8>
   b67ff:	cmpq   $0x1,0x10(%rsp)
   b6805:	jne    b6839 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x519>
   b6807:	lea    0x5b0(%rsp),%rdi
   b680f:	lea    0x100(%rsp),%rsi
   b6817:	mov    0x8(%rsp),%rdx
   b681c:	mov    0x40(%rsp),%rcx
   b6821:	mov    0x50(%rsp),%r8
   b6826:	call   b61c0 <rvvdk_vmdk::stream_map::Budget<S>::read>
   b682b:	cmpl   $0x10,0x5b0(%rsp)
   b6833:	jne    b6918 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x5f8>
   b6839:	mov    0x78(%rsp),%rax
   b683e:	lea    0x0(,%rax,4),%rdi
   b6846:	cmp    %r13,%rdi
   b6849:	mov    0x50(%rsp),%rsi
   b684e:	ja     b6f73 <rvvdk_vmdk::stream_map::StreamMap::read_from+0xc53>
   b6854:	shrq   $0x10,0xd0(%rsp)
   b685d:	lea    (%r14,%rdi,1),%rax
   b6861:	mov    %rdi,%rcx
   b6864:	cmp    %rcx,%r13
   b6867:	je     b69be <rvvdk_vmdk::stream_map::StreamMap::read_from+0x69e>
   b686d:	cmpb   $0x0,(%r14,%rcx,1)
   b6872:	lea    0x1(%rcx),%rcx
   b6876:	je     b6864 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x544>
   b6878:	jmp    b69e3 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x6c3>
   b687d:	lea    0x5b0(%rsp),%rdi
   b6885:	mov    $0x1,%edx
   b688a:	mov    $0x1,%r8d
   b6890:	mov    $0x1,%r9d
   b6896:	xor    %esi,%esi
   b6898:	mov    %r14,%rcx
   b689b:	call   ce6f0 <alloc::raw_vec::RawVecInner<A>::finish_grow>
   b68a0:	cmpb   $0x0,0x5b0(%rsp)
   b68a8:	je     b6937 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x617>
   b68ae:	mov    %r14,0x10(%r15)
   b68b2:	movq   $0xd,0x8(%r15)
   b68ba:	movq   $0x3,(%r15)
   b68c1:	jmp    b6b85 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x865>
   b68c6:	lea    0x5b0(%rsp),%rdi
   b68ce:	mov    $0x1,%edx
   b68d3:	mov    $0x1,%r8d
   b68d9:	mov    $0x1,%r9d
   b68df:	xor    %esi,%esi
   b68e1:	mov    %rcx,%r14
   b68e4:	call   ce6f0 <alloc::raw_vec::RawVecInner<A>::finish_grow>
   b68e9:	cmpb   $0x0,0x5b0(%rsp)
   b68f1:	je     b6978 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x658>
   b68f7:	mov    %r14,0x10(%r15)
   b68fb:	movq   $0xd,0x8(%r15)
   b6903:	movq   $0x3,(%r15)
   b690a:	mov    (%rsp),%r13
   b690e:	mov    0x18(%rsp),%r14
   b6913:	jmp    b6b6f <rvvdk_vmdk::stream_map::StreamMap::read_from+0x84f>
   b6918:	movups 0x5b0(%rsp),%xmm0
   b6920:	movups 0x5c0(%rsp),%xmm1
   b6928:	movups %xmm1,0x18(%r15)
   b692d:	movups %xmm0,0x8(%r15)
   b6932:	jmp    b6b4e <rvvdk_vmdk::stream_map::StreamMap::read_from+0x82e>
   b6937:	mov    0x5b8(%rsp),%rax
   b693f:	mov    %rax,0x18(%rsp)
   b6944:	cmp    $0x1,%r14
   b6948:	je     b696c <rvvdk_vmdk::stream_map::StreamMap::read_from+0x64c>
   b694a:	mov    (%rsp),%rax
   b694e:	lea    -0x1(%rax),%rdx
   b6952:	mov    0x18(%rsp),%r14
   b6957:	mov    %r14,%rdi
   b695a:	xor    %esi,%esi
   b695c:	call   *0x2101d6(%rip)        # 2c6b38 <memset@GLIBC_2.2.5>
   b6962:	mov    (%rsp),%rax
   b6966:	add    %r14,%rax
   b6969:	dec    %rax
   b696c:	movb   $0x0,(%rax)
   b696f:	mov    (%rsp),%r14
   b6973:	jmp    b66a2 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x382>
   b6978:	mov    0x5b8(%rsp),%rax
   b6980:	mov    %rax,0x40(%rsp)
   b6985:	cmp    $0x1,%r14
   b6989:	je     b69af <rvvdk_vmdk::stream_map::StreamMap::read_from+0x68f>
   b698b:	mov    0x60(%rsp),%rax
   b6990:	lea    -0x1(%rax),%rdx
   b6994:	mov    0x40(%rsp),%r14
   b6999:	mov    %r14,%rdi
   b699c:	xor    %esi,%esi
   b699e:	call   *0x210194(%rip)        # 2c6b38 <memset@GLIBC_2.2.5>
   b69a4:	mov    0x60(%rsp),%rax
   b69a9:	add    %r14,%rax
   b69ac:	dec    %rax
   b69af:	movb   $0x0,(%rax)
   b69b2:	mov    (%rsp),%r14
   b69b6:	mov    %r14,%rax
   b69b9:	jmp    b66c0 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x3a0>
   b69be:	test   %rsi,%rsi
   b69c1:	je     b6a03 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x6e3>
   b69c3:	cmp    %rsi,%rdi
   b69c6:	ja     b71df <rvvdk_vmdk::stream_map::StreamMap::read_from+0xebf>
   b69cc:	mov    %rdi,%rcx
   b69cf:	mov    0x40(%rsp),%rdx
   b69d4:	cmp    %rcx,%rsi
   b69d7:	je     b6a03 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x6e3>
   b69d9:	cmpb   $0x0,(%rdx,%rcx,1)
   b69dd:	lea    0x1(%rcx),%rcx
   b69e1:	je     b69d4 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x6b4>
   b69e3:	movq   $0xa,0x8(%r15)
   b69eb:	lea    -0x961ef(%rip),%rax        # 20803 <anon.321e7b27d4937aed77e095c38e205854.42.llvm.3789170322229662815+0x21a>
   b69f2:	mov    %rax,0x10(%r15)
   b69f6:	movq   $0x16,0x18(%r15)
   b69fe:	jmp    b6b4e <rvvdk_vmdk::stream_map::StreamMap::read_from+0x82e>
   b6a03:	mov    %r14,0x5b0(%rsp)
   b6a0b:	mov    %rdi,0x5b8(%rsp)
   b6a13:	mov    %rax,0x5c0(%rsp)
   b6a1b:	movq   $0x0,0x5c8(%rsp)
   b6a27:	movq   $0x4,0x5d0(%rsp)
   b6a33:	lea    0x5b0(%rsp),%rdi
   b6a3b:	call   b5dc0 <<core::iter::adapters::filter::Filter<I,P> as core::iter::traits::iterator::Iterator>::count>
   b6a40:	mov    %rax,0x38(%rsp)
   b6a45:	lea    0x5b0(%rsp),%rdi
   b6a4d:	mov    0x38(%rsp),%rsi
   b6a52:	mov    0x20(%rsp),%rdx
   b6a57:	call   *0x21026b(%rip)        # 2c6cc8 <_DYNAMIC+0x618>
   b6a5d:	mov    0x5b0(%rsp),%rax
   b6a65:	mov    0x5b8(%rsp),%rsi
   b6a6d:	cmp    $0x10,%rax
   b6a71:	jne    b6b39 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x819>
   b6a77:	lea    0xdb0(%rsp),%rdi
   b6a7f:	mov    $0x4,%edx
   b6a84:	call   *0x210236(%rip)        # 2c6cc0 <_DYNAMIC+0x610>
   b6a8a:	mov    0xdb0(%rsp),%rax
   b6a92:	mov    0xdb8(%rsp),%r12
   b6a9a:	cmp    $0x10,%rax
   b6a9e:	jne    b6b9a <rvvdk_vmdk::stream_map::StreamMap::read_from+0x87a>
   b6aa4:	lea    0x5b0(%rsp),%rdi
   b6aac:	mov    $0x10,%edx
   b6ab1:	mov    %r12,%rsi
   b6ab4:	call   *0x21020e(%rip)        # 2c6cc8 <_DYNAMIC+0x618>
   b6aba:	mov    0x5b0(%rsp),%rax
   b6ac2:	mov    0x5b8(%rsp),%rdx
   b6aca:	cmp    $0x10,%rax
   b6ace:	jne    b6bb1 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x891>
   b6ad4:	lea    0xdb0(%rsp),%rdi
   b6adc:	mov    0x58(%rsp),%rsi
   b6ae1:	call   *0x2101d9(%rip)        # 2c6cc0 <_DYNAMIC+0x610>
   b6ae7:	mov    0xdb0(%rsp),%rax
   b6aef:	mov    0xdb8(%rsp),%rcx
   b6af7:	mov    %rcx,0x58(%rsp)
   b6afc:	cmp    $0x10,%rax
   b6b00:	jne    b6bbe <rvvdk_vmdk::stream_map::StreamMap::read_from+0x89e>
   b6b06:	mov    0x58(%rsp),%rax
   b6b0b:	cmp    0xa0(%rsp),%rax
   b6b13:	jbe    b6bfb <rvvdk_vmdk::stream_map::StreamMap::read_from+0x8db>
   b6b19:	movq   $0xb,0x8(%r15)
   b6b21:	lea    -0x9f828(%rip),%rax        # 17300 <anon.d625489d584c397ac22d75864c33158f.35.llvm.5936746164385555759+0x1a0>
   b6b28:	mov    %rax,0x10(%r15)
   b6b2c:	movq   $0x10,0x18(%r15)
   b6b34:	jmp    b6bd8 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x8b8>
   b6b39:	movups 0x5c0(%rsp),%xmm0
   b6b41:	movups %xmm0,0x18(%r15)
   b6b46:	mov    %rax,0x8(%r15)
   b6b4a:	mov    %rsi,0x10(%r15)
   b6b4e:	movq   $0x3,(%r15)
   b6b55:	mov    0x60(%rsp),%rsi
   b6b5a:	test   %rsi,%rsi
   b6b5d:	je     b6b6f <rvvdk_vmdk::stream_map::StreamMap::read_from+0x84f>
   b6b5f:	mov    $0x1,%edx
   b6b64:	mov    0x40(%rsp),%rdi
   b6b69:	call   *0x20fd91(%rip)        # 2c6900 <_DYNAMIC+0x250>
   b6b6f:	test   %r13,%r13
   b6b72:	je     b6b85 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x865>
   b6b74:	mov    $0x1,%edx
   b6b79:	mov    %r14,%rdi
   b6b7c:	mov    %r13,%rsi
   b6b7f:	call   *0x20fd7b(%rip)        # 2c6900 <_DYNAMIC+0x250>
   b6b85:	mov    %r15,%rax
   b6b88:	add    $0x15b8,%rsp
   b6b8f:	pop    %rbx
   b6b90:	pop    %r12
   b6b92:	pop    %r13
   b6b94:	pop    %r14
   b6b96:	pop    %r15
   b6b98:	pop    %rbp
   b6b99:	ret
   b6b9a:	movups 0xdc0(%rsp),%xmm0
   b6ba2:	movups %xmm0,0x18(%r15)
   b6ba7:	mov    %rax,0x8(%r15)
   b6bab:	mov    %r12,0x10(%r15)
   b6baf:	jmp    b6b4e <rvvdk_vmdk::stream_map::StreamMap::read_from+0x82e>
   b6bb1:	movups 0x5c0(%rsp),%xmm0
   b6bb9:	jmp    b67a2 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x482>
   b6bbe:	movups 0xdc0(%rsp),%xmm0
   b6bc6:	movups %xmm0,0x18(%r15)
   b6bcb:	mov    %rax,0x8(%r15)
   b6bcf:	mov    0x58(%rsp),%rax
   b6bd4:	mov    %rax,0x10(%r15)
   b6bd8:	movq   $0x3,(%r15)
   b6bdf:	mov    (%rsp),%r13
   b6be3:	mov    0x18(%rsp),%r14
   b6be8:	mov    0x60(%rsp),%rsi
   b6bed:	test   %rsi,%rsi
   b6bf0:	jne    b6b5f <rvvdk_vmdk::stream_map::StreamMap::read_from+0x83f>
   b6bf6:	jmp    b6b6f <rvvdk_vmdk::stream_map::StreamMap::read_from+0x84f>
   b6bfb:	lea    0x5b0(%rsp),%rdi
   b6c03:	mov    %r12,%rsi
   b6c06:	call   b8350 <rvvdk_vmdk::stream_map::allocated>
   b6c0b:	mov    0x5b0(%rsp),%rax
   b6c13:	movups 0x5b8(%rsp),%xmm0
   b6c1b:	movaps %xmm0,0xdb0(%rsp)
   b6c23:	mov    0x5c8(%rsp),%rcx
   b6c2b:	mov    %rcx,0xdc0(%rsp)
   b6c33:	cmp    $0x10,%rax
   b6c37:	jne    b6d98 <rvvdk_vmdk::stream_map::StreamMap::read_from+0xa78>
   b6c3d:	mov    0xdc0(%rsp),%rax
   b6c45:	mov    %rax,0xc0(%rsp)
   b6c4d:	movaps 0xdb0(%rsp),%xmm0
   b6c55:	movaps %xmm0,0xb0(%rsp)
   b6c5d:	movq   $0x0,0xd8(%rsp)
   b6c69:	lea    0x88(%rsp),%rax
   b6c71:	mov    %rax,0xe8(%rsp)
   b6c79:	lea    0xb0(%rsp),%rax
   b6c81:	mov    %rax,0xf0(%rsp)
   b6c89:	lea    0xd8(%rsp),%rax
   b6c91:	mov    %rax,0xf8(%rsp)
   b6c99:	lea    0x5b0(%rsp),%rdi
   b6ca1:	lea    0xe8(%rsp),%rsi
   b6ca9:	mov    $0x200,%ecx
   b6cae:	xor    %edx,%edx
   b6cb0:	call   b8290 <rvvdk_vmdk::stream_map::StreamMap::read_from::{{closure}}>
   b6cb5:	cmpl   $0x10,0x5b0(%rsp)
   b6cbd:	jne    b7109 <rvvdk_vmdk::stream_map::StreamMap::read_from+0xde9>
   b6cc3:	mov    0x288(%rsp),%rdx
   b6ccb:	mov    0x290(%rsp),%rcx
   b6cd3:	lea    0x5b0(%rsp),%rdi
   b6cdb:	lea    0xe8(%rsp),%rsi
   b6ce3:	call   b8290 <rvvdk_vmdk::stream_map::StreamMap::read_from::{{closure}}>
   b6ce8:	cmpl   $0x10,0x5b0(%rsp)
   b6cf0:	jne    b7109 <rvvdk_vmdk::stream_map::StreamMap::read_from+0xde9>
   b6cf6:	lea    0x5b0(%rsp),%rdi
   b6cfe:	lea    0xe8(%rsp),%rsi
   b6d06:	mov    0x80(%rsp),%rdx
   b6d0e:	mov    (%rsp),%rcx
   b6d12:	call   b8290 <rvvdk_vmdk::stream_map::StreamMap::read_from::{{closure}}>
   b6d17:	cmpl   $0x10,0x5b0(%rsp)
   b6d1f:	jne    b7109 <rvvdk_vmdk::stream_map::StreamMap::read_from+0xde9>
   b6d25:	testb  $0x1,0x10(%rsp)
   b6d2a:	je     b6d59 <rvvdk_vmdk::stream_map::StreamMap::read_from+0xa39>
   b6d2c:	lea    0x5b0(%rsp),%rdi
   b6d34:	lea    0xe8(%rsp),%rsi
   b6d3c:	mov    0x8(%rsp),%rdx
   b6d41:	mov    0x30(%rsp),%rcx
   b6d46:	call   b8290 <rvvdk_vmdk::stream_map::StreamMap::read_from::{{closure}}>
   b6d4b:	cmpl   $0x10,0x5b0(%rsp)
   b6d53:	jne    b7109 <rvvdk_vmdk::stream_map::StreamMap::read_from+0xde9>
   b6d59:	cmpq   $0x0,0x78(%rsp)
   b6d5f:	je     b7157 <rvvdk_vmdk::stream_map::StreamMap::read_from+0xe37>
   b6d65:	mov    0x298(%rsp),%rax
   b6d6d:	mov    %rax,0x8(%rsp)
   b6d72:	cmpl   $0x2,0x98(%rsp)
   b6d7a:	jne    b6f84 <rvvdk_vmdk::stream_map::StreamMap::read_from+0xc64>
   b6d80:	mov    0x80(%rsp),%rax
   b6d88:	add    $0xfffffffffffffe00,%rax
   b6d8e:	mov    %rax,0x28(%rsp)
   b6d93:	xor    %r14d,%r14d
   b6d96:	jmp    b6dce <rvvdk_vmdk::stream_map::StreamMap::read_from+0xaae>
   b6d98:	mov    0xdc0(%rsp),%rcx
   b6da0:	mov    %rcx,0x20(%r15)
   b6da4:	movaps 0xdb0(%rsp),%xmm0
   b6dac:	movups %xmm0,0x10(%r15)
   b6db1:	mov    %rax,0x8(%r15)
   b6db5:	jmp    b6bd8 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x8b8>
   b6dba:	mov    %r14,%rax
   b6dbd:	inc    %rax
   b6dc0:	mov    %rax,%r14
   b6dc3:	cmp    %rax,0x78(%rsp)
   b6dc8:	je     b7157 <rvvdk_vmdk::stream_map::StreamMap::read_from+0xe37>
   b6dce:	mov    0x18(%rsp),%rdi
   b6dd3:	mov    (%rsp),%rsi
   b6dd7:	mov    %r14,%rdx
   b6dda:	call   *0x20fed8(%rip)        # 2c6cb8 <_DYNAMIC+0x608>
   b6de0:	mov    %eax,%r13d
   b6de3:	xor    %r12d,%r12d
   b6de6:	cmpl   $0x1,0x10(%rsp)
   b6deb:	jne    b6e17 <rvvdk_vmdk::stream_map::StreamMap::read_from+0xaf7>
   b6ded:	mov    0x40(%rsp),%rdi
   b6df2:	mov    0x50(%rsp),%rsi
   b6df7:	mov    %r14,%rdx
   b6dfa:	call   *0x20feb8(%rip)        # 2c6cb8 <_DYNAMIC+0x608>
   b6e00:	mov    %eax,%r12d
   b6e03:	test   %r13d,%r13d
   b6e06:	sete   %al
   b6e09:	test   %r12d,%r12d
   b6e0c:	sete   %cl
   b6e0f:	xor    %al,%cl
   b6e11:	jne    b72b1 <rvvdk_vmdk::stream_map::StreamMap::read_from+0xf91>
   b6e17:	test   %r13d,%r13d
   b6e1a:	je     b6e30 <rvvdk_vmdk::stream_map::StreamMap::read_from+0xb10>
   b6e1c:	cmp    $0x1,%r13d
   b6e20:	je     b726f <rvvdk_vmdk::stream_map::StreamMap::read_from+0xf4f>
   b6e26:	movl   $0x0,0x30(%rsp)
   b6e2e:	jmp    b6e48 <rvvdk_vmdk::stream_map::StreamMap::read_from+0xb28>
   b6e30:	test   %r12d,%r12d
   b6e33:	je     b6dba <rvvdk_vmdk::stream_map::StreamMap::read_from+0xa9a>
   b6e35:	mov    $0x1,%al
   b6e37:	mov    %eax,0x30(%rsp)
   b6e3b:	mov    %r12d,%r13d
   b6e3e:	cmp    $0x1,%r12d
   b6e42:	je     b726f <rvvdk_vmdk::stream_map::StreamMap::read_from+0xf4f>
   b6e48:	mov    %r13d,%r13d
   b6e4b:	shl    $0x9,%r13
   b6e4f:	mov    $0x800,%edx
   b6e54:	lea    0x5b0(%rsp),%rdi
   b6e5c:	mov    %r13,%rsi
   b6e5f:	call   *0x20fe5b(%rip)        # 2c6cc0 <_DYNAMIC+0x610>
   b6e65:	mov    0x5b0(%rsp),%rax
   b6e6d:	cmp    $0x10,%rax
   b6e71:	jne    b728f <rvvdk_vmdk::stream_map::StreamMap::read_from+0xf6f>
   b6e77:	add    $0xfffffffffffffe00,%r13
   b6e7e:	cmp    0x8(%rsp),%r13
   b6e83:	jb     b72dc <rvvdk_vmdk::stream_map::StreamMap::read_from+0xfbc>
   b6e89:	mov    0x5b8(%rsp),%rax
   b6e91:	mov    %rax,0x8(%rsp)
   b6e96:	cmp    0x28(%rsp),%rax
   b6e9b:	ja     b72dc <rvvdk_vmdk::stream_map::StreamMap::read_from+0xfbc>
   b6ea1:	mov    $0xa00,%ecx
   b6ea6:	lea    0x5b0(%rsp),%rdi
   b6eae:	lea    0xe8(%rsp),%rsi
   b6eb6:	mov    %r13,%rdx
   b6eb9:	call   b8290 <rvvdk_vmdk::stream_map::StreamMap::read_from::{{closure}}>
   b6ebe:	cmpl   $0x10,0x5b0(%rsp)
   b6ec6:	jne    b7109 <rvvdk_vmdk::stream_map::StreamMap::read_from+0xde9>
   b6ecc:	cmpb   $0x0,0x30(%rsp)
   b6ed1:	jne    b6dba <rvvdk_vmdk::stream_map::StreamMap::read_from+0xa9a>
   b6ed7:	test   %r12d,%r12d
   b6eda:	je     b6dba <rvvdk_vmdk::stream_map::StreamMap::read_from+0xa9a>
   b6ee0:	cmp    $0x1,%r12d
   b6ee4:	je     b726f <rvvdk_vmdk::stream_map::StreamMap::read_from+0xf4f>
   b6eea:	mov    %r12d,%r12d
   b6eed:	shl    $0x9,%r12
   b6ef1:	mov    $0x800,%edx
   b6ef6:	lea    0x5b0(%rsp),%rdi
   b6efe:	mov    %r12,%rsi
   b6f01:	call   *0x20fdb9(%rip)        # 2c6cc0 <_DYNAMIC+0x610>
   b6f07:	mov    0x5b0(%rsp),%rax
   b6f0f:	cmp    $0x10,%rax
   b6f13:	jne    b728f <rvvdk_vmdk::stream_map::StreamMap::read_from+0xf6f>
   b6f19:	add    $0xfffffffffffffe00,%r12
   b6f20:	cmp    0x8(%rsp),%r12
   b6f25:	jb     b72dc <rvvdk_vmdk::stream_map::StreamMap::read_from+0xfbc>
   b6f2b:	mov    0x5b8(%rsp),%rax
   b6f33:	mov    %rax,0x8(%rsp)
   b6f38:	cmp    0x28(%rsp),%rax
   b6f3d:	ja     b72dc <rvvdk_vmdk::stream_map::StreamMap::read_from+0xfbc>
   b6f43:	mov    $0xa00,%ecx
   b6f48:	lea    0x5b0(%rsp),%rdi
   b6f50:	lea    0xe8(%rsp),%rsi
   b6f58:	mov    %r12,%rdx
   b6f5b:	call   b8290 <rvvdk_vmdk::stream_map::StreamMap::read_from::{{closure}}>
   b6f60:	cmpl   $0x10,0x5b0(%rsp)
   b6f68:	je     b6dba <rvvdk_vmdk::stream_map::StreamMap::read_from+0xa9a>
   b6f6e:	jmp    b7109 <rvvdk_vmdk::stream_map::StreamMap::read_from+0xde9>
   b6f73:	lea    0x202bde(%rip),%rcx        # 2b9b58 <anon.321e7b27d4937aed77e095c38e205854.44.llvm.3789170322229662815+0x78>
   b6f7a:	mov    %r13,0x50(%rsp)
   b6f7f:	jmp    b71e6 <rvvdk_vmdk::stream_map::StreamMap::read_from+0xec6>
   b6f84:	xor    %r12d,%r12d
   b6f87:	jmp    b6f97 <rvvdk_vmdk::stream_map::StreamMap::read_from+0xc77>
   b6f89:	inc    %r12
   b6f8c:	cmp    %r12,0x78(%rsp)
   b6f91:	je     b7157 <rvvdk_vmdk::stream_map::StreamMap::read_from+0xe37>
   b6f97:	mov    0x18(%rsp),%rdi
   b6f9c:	mov    (%rsp),%rsi
   b6fa0:	mov    %r12,%rdx
   b6fa3:	call   *0x20fd0f(%rip)        # 2c6cb8 <_DYNAMIC+0x608>
   b6fa9:	mov    %eax,%r13d
   b6fac:	xor    %r14d,%r14d
   b6faf:	cmpl   $0x1,0x10(%rsp)
   b6fb4:	jne    b6fe0 <rvvdk_vmdk::stream_map::StreamMap::read_from+0xcc0>
   b6fb6:	mov    0x40(%rsp),%rdi
   b6fbb:	mov    0x50(%rsp),%rsi
   b6fc0:	mov    %r12,%rdx
   b6fc3:	call   *0x20fcef(%rip)        # 2c6cb8 <_DYNAMIC+0x608>
   b6fc9:	mov    %eax,%r14d
   b6fcc:	test   %r13d,%r13d
   b6fcf:	sete   %al
   b6fd2:	test   %r14d,%r14d
   b6fd5:	sete   %cl
   b6fd8:	xor    %al,%cl
   b6fda:	jne    b72b1 <rvvdk_vmdk::stream_map::StreamMap::read_from+0xf91>
   b6fe0:	test   %r13d,%r13d
   b6fe3:	je     b6ff9 <rvvdk_vmdk::stream_map::StreamMap::read_from+0xcd9>
   b6fe5:	cmp    $0x1,%r13d
   b6fe9:	je     b726f <rvvdk_vmdk::stream_map::StreamMap::read_from+0xf4f>
   b6fef:	movl   $0x0,0x30(%rsp)
   b6ff7:	jmp    b7011 <rvvdk_vmdk::stream_map::StreamMap::read_from+0xcf1>
   b6ff9:	test   %r14d,%r14d
   b6ffc:	je     b6f89 <rvvdk_vmdk::stream_map::StreamMap::read_from+0xc69>
   b6ffe:	mov    $0x1,%al
   b7000:	mov    %eax,0x30(%rsp)
   b7004:	mov    %r14d,%r13d
   b7007:	cmp    $0x1,%r14d
   b700b:	je     b726f <rvvdk_vmdk::stream_map::StreamMap::read_from+0xf4f>
   b7011:	mov    %r13d,%r13d
   b7014:	shl    $0x9,%r13
   b7018:	mov    $0x800,%edx
   b701d:	lea    0x5b0(%rsp),%rdi
   b7025:	mov    %r13,%rsi
   b7028:	call   *0x20fc92(%rip)        # 2c6cc0 <_DYNAMIC+0x610>
   b702e:	mov    0x5b0(%rsp),%rax
   b7036:	cmp    $0x10,%rax
   b703a:	jne    b728f <rvvdk_vmdk::stream_map::StreamMap::read_from+0xf6f>
   b7040:	mov    0x8(%rsp),%rax
   b7045:	cmp    %rax,0x5b8(%rsp)
   b704d:	ja     b72fc <rvvdk_vmdk::stream_map::StreamMap::read_from+0xfdc>
   b7053:	mov    $0x800,%ecx
   b7058:	lea    0x5b0(%rsp),%rdi
   b7060:	lea    0xe8(%rsp),%rsi
   b7068:	mov    %r13,%rdx
   b706b:	call   b8290 <rvvdk_vmdk::stream_map::StreamMap::read_from::{{closure}}>
   b7070:	cmpl   $0x10,0x5b0(%rsp)
   b7078:	jne    b7109 <rvvdk_vmdk::stream_map::StreamMap::read_from+0xde9>
   b707e:	cmpb   $0x0,0x30(%rsp)
   b7083:	jne    b6f89 <rvvdk_vmdk::stream_map::StreamMap::read_from+0xc69>
   b7089:	test   %r14d,%r14d
   b708c:	je     b6f89 <rvvdk_vmdk::stream_map::StreamMap::read_from+0xc69>
   b7092:	cmp    $0x1,%r14d
   b7096:	je     b726f <rvvdk_vmdk::stream_map::StreamMap::read_from+0xf4f>
   b709c:	mov    %r14d,%r13d
   b709f:	shl    $0x9,%r13
   b70a3:	mov    $0x800,%edx
   b70a8:	lea    0x5b0(%rsp),%rdi
   b70b0:	mov    %r13,%rsi
   b70b3:	call   *0x20fc07(%rip)        # 2c6cc0 <_DYNAMIC+0x610>
   b70b9:	mov    0x5b0(%rsp),%rax
   b70c1:	cmp    $0x10,%rax
   b70c5:	jne    b728f <rvvdk_vmdk::stream_map::StreamMap::read_from+0xf6f>
   b70cb:	mov    0x8(%rsp),%rax
   b70d0:	cmp    %rax,0x5b8(%rsp)
   b70d8:	ja     b72fc <rvvdk_vmdk::stream_map::StreamMap::read_from+0xfdc>
   b70de:	mov    $0x800,%ecx
   b70e3:	lea    0x5b0(%rsp),%rdi
   b70eb:	lea    0xe8(%rsp),%rsi
   b70f3:	mov    %r13,%rdx
   b70f6:	call   b8290 <rvvdk_vmdk::stream_map::StreamMap::read_from::{{closure}}>
   b70fb:	cmpl   $0x10,0x5b0(%rsp)
   b7103:	je     b6f89 <rvvdk_vmdk::stream_map::StreamMap::read_from+0xc69>
   b7109:	movups 0x5b0(%rsp),%xmm0
   b7111:	movups 0x5c0(%rsp),%xmm1
   b7119:	movups %xmm1,0x18(%r15)
   b711e:	movups %xmm0,0x8(%r15)
   b7123:	movq   $0x3,(%r15)
   b712a:	mov    0xb0(%rsp),%rsi
   b7132:	test   %rsi,%rsi
   b7135:	je     b6bdf <rvvdk_vmdk::stream_map::StreamMap::read_from+0x8bf>
   b713b:	mov    0xb8(%rsp),%rdi
   b7143:	shl    $0x4,%rsi
   b7147:	mov    $0x8,%edx
   b714c:	call   *0x20f7ae(%rip)        # 2c6900 <_DYNAMIC+0x250>
   b7152:	jmp    b6bdf <rvvdk_vmdk::stream_map::StreamMap::read_from+0x8bf>
   b7157:	mov    0xd8(%rsp),%rsi
   b715f:	mov    0xc0(%rsp),%rdx
   b7167:	cmp    %rdx,%rsi
   b716a:	ja     b7505 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x11e5>
   b7170:	mov    0xb8(%rsp),%rdi
   b7178:	call   b85a0 <core::slice::<impl [T]>::sort_unstable_by_key>
   b717d:	mov    0xd8(%rsp),%rsi
   b7185:	mov    0xc0(%rsp),%rdx
   b718d:	cmp    %rdx,%rsi
   b7190:	ja     b750e <rvvdk_vmdk::stream_map::StreamMap::read_from+0x11ee>
   b7196:	mov    0xb8(%rsp),%rax
   b719e:	mov    %rax,0x5b0(%rsp)
   b71a6:	mov    %rsi,0x5b8(%rsp)
   b71ae:	movq   $0x2,0x5c0(%rsp)
   b71ba:	lea    0x5b0(%rsp),%rdi
   b71c2:	call   b8530 <core::iter::traits::iterator::Iterator::try_fold>
   b71c7:	test   %al,%al
   b71c9:	je     b71f9 <rvvdk_vmdk::stream_map::StreamMap::read_from+0xed9>
   b71cb:	movq   $0xa,0x8(%r15)
   b71d3:	lea    -0x96a4d(%rip),%rax        # 2078d <anon.321e7b27d4937aed77e095c38e205854.42.llvm.3789170322229662815+0x1a4>
   b71da:	jmp    b727e <rvvdk_vmdk::stream_map::StreamMap::read_from+0xf5e>
   b71df:	lea    0x20295a(%rip),%rcx        # 2b9b40 <anon.321e7b27d4937aed77e095c38e205854.44.llvm.3789170322229662815+0x60>
   b71e6:	mov    0x50(%rsp),%rdx
   b71eb:	mov    %rdx,%rsi
   b71ee:	call   *0x20f93c(%rip)        # 2c6b30 <_DYNAMIC+0x480>
   b71f4:	jmp    b81cc <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1eac>
   b71f9:	lea    0x5b0(%rsp),%rdi
   b7201:	mov    $0x200,%edx
   b7206:	mov    0x38(%rsp),%rsi
   b720b:	call   *0x20fab7(%rip)        # 2c6cc8 <_DYNAMIC+0x618>
   b7211:	mov    0x5b0(%rsp),%rax
   b7219:	mov    0x5b8(%rsp),%rsi
   b7221:	cmp    $0x10,%rax
   b7225:	jne    b73f6 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x10d6>
   b722b:	lea    0xdb0(%rsp),%rdi
   b7233:	mov    $0x2,%edx
   b7238:	call   *0x20fa8a(%rip)        # 2c6cc8 <_DYNAMIC+0x618>
   b723e:	mov    0xdb0(%rsp),%rax
   b7246:	mov    0xdb8(%rsp),%r13
   b724e:	cmp    $0x10,%rax
   b7252:	jne    b72c2 <rvvdk_vmdk::stream_map::StreamMap::read_from+0xfa2>
   b7254:	cmp    0x68(%rbp),%r13
   b7258:	jbe    b731c <rvvdk_vmdk::stream_map::StreamMap::read_from+0xffc>
   b725e:	movq   $0xb,0x8(%r15)
   b7266:	lea    -0x96bee(%rip),%rax        # 2067f <anon.321e7b27d4937aed77e095c38e205854.42.llvm.3789170322229662815+0x96>
   b726d:	jmp    b727e <rvvdk_vmdk::stream_map::StreamMap::read_from+0xf5e>
   b726f:	movq   $0xa,0x8(%r15)
   b7277:	lea    -0x96ac0(%rip),%rax        # 207be <anon.321e7b27d4937aed77e095c38e205854.42.llvm.3789170322229662815+0x1d5>
   b727e:	mov    %rax,0x10(%r15)
   b7282:	movq   $0x16,0x18(%r15)
   b728a:	jmp    b7123 <rvvdk_vmdk::stream_map::StreamMap::read_from+0xe03>
   b728f:	mov    0x5b8(%rsp),%rcx
   b7297:	movups 0x5c0(%rsp),%xmm0
   b729f:	movups %xmm0,0x18(%r15)
   b72a4:	mov    %rax,0x8(%r15)
   b72a8:	mov    %rcx,0x10(%r15)
   b72ac:	jmp    b7123 <rvvdk_vmdk::stream_map::StreamMap::read_from+0xe03>
   b72b1:	movq   $0xa,0x8(%r15)
   b72b9:	lea    -0x96b1d(%rip),%rax        # 207a3 <anon.321e7b27d4937aed77e095c38e205854.42.llvm.3789170322229662815+0x1ba>
   b72c0:	jmp    b72eb <rvvdk_vmdk::stream_map::StreamMap::read_from+0xfcb>
   b72c2:	movups 0xdc0(%rsp),%xmm0
   b72ca:	movups %xmm0,0x18(%r15)
   b72cf:	mov    %rax,0x8(%r15)
   b72d3:	mov    %r13,0x10(%r15)
   b72d7:	jmp    b7123 <rvvdk_vmdk::stream_map::StreamMap::read_from+0xe03>
   b72dc:	movq   $0xa,0x8(%r15)
   b72e4:	lea    -0x96b03(%rip),%rax        # 207e8 <anon.321e7b27d4937aed77e095c38e205854.42.llvm.3789170322229662815+0x1ff>
   b72eb:	mov    %rax,0x10(%r15)
   b72ef:	movq   $0x1b,0x18(%r15)
   b72f7:	jmp    b7123 <rvvdk_vmdk::stream_map::StreamMap::read_from+0xe03>
   b72fc:	movq   $0xa,0x8(%r15)
   b7304:	lea    -0x96b37(%rip),%rax        # 207d4 <anon.321e7b27d4937aed77e095c38e205854.42.llvm.3789170322229662815+0x1eb>
   b730b:	mov    %rax,0x10(%r15)
   b730f:	movq   $0x14,0x18(%r15)
   b7317:	jmp    b7123 <rvvdk_vmdk::stream_map::StreamMap::read_from+0xe03>
   b731c:	mov    0x110(%rsp),%r12
   b7324:	lea    0x5b0(%rsp),%rdi
   b732c:	mov    $0x800,%edx
   b7331:	mov    0x38(%rsp),%rsi
   b7336:	call   *0x20f98c(%rip)        # 2c6cc8 <_DYNAMIC+0x618>
   b733c:	mov    0x5b0(%rsp),%rax
   b7344:	mov    0x5b8(%rsp),%rsi
   b734c:	cmp    $0x10,%rax
   b7350:	jne    b73f6 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x10d6>
   b7356:	lea    0xdb0(%rsp),%rdi
   b735e:	mov    0x20(%rsp),%rdx
   b7363:	call   *0x20f95f(%rip)        # 2c6cc8 <_DYNAMIC+0x618>
   b7369:	mov    0xdb0(%rsp),%rax
   b7371:	mov    0xdb8(%rsp),%rsi
   b7379:	cmp    $0x10,%rax
   b737d:	jne    b7410 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x10f0>
   b7383:	lea    0x350(%rsp),%rdi
   b738b:	mov    $0x2,%edx
   b7390:	call   *0x20f932(%rip)        # 2c6cc8 <_DYNAMIC+0x618>
   b7396:	mov    0x350(%rsp),%rax
   b739e:	mov    0x358(%rsp),%r14
   b73a6:	cmp    $0x10,%rax
   b73aa:	jne    b741a <rvvdk_vmdk::stream_map::StreamMap::read_from+0x10fa>
   b73ac:	cmpl   $0x2,0x98(%rsp)
   b73b4:	jne    b7434 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1114>
   b73b6:	lea    0x5b0(%rsp),%rdi
   b73be:	mov    $0x10,%edx
   b73c3:	mov    0x38(%rsp),%rsi
   b73c8:	call   *0x20f8fa(%rip)        # 2c6cc8 <_DYNAMIC+0x618>
   b73ce:	mov    0x5b0(%rsp),%rax
   b73d6:	mov    0x5b8(%rsp),%rdx
   b73de:	cmp    $0x10,%rax
   b73e2:	je     b7436 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1116>
   b73e4:	movups 0x5c0(%rsp),%xmm0
   b73ec:	movups %xmm0,0x18(%r15)
   b73f1:	mov    %rdx,%r14
   b73f4:	jmp    b7427 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1107>
   b73f6:	movups 0x5c0(%rsp),%xmm0
   b73fe:	movups %xmm0,0x18(%r15)
   b7403:	mov    %rax,0x8(%r15)
   b7407:	mov    %rsi,0x10(%r15)
   b740b:	jmp    b7123 <rvvdk_vmdk::stream_map::StreamMap::read_from+0xe03>
   b7410:	movups 0xdc0(%rsp),%xmm0
   b7418:	jmp    b73fe <rvvdk_vmdk::stream_map::StreamMap::read_from+0x10de>
   b741a:	movups 0x360(%rsp),%xmm0
   b7422:	movups %xmm0,0x18(%r15)
   b7427:	mov    %rax,0x8(%r15)
   b742b:	mov    %r14,0x10(%r15)
   b742f:	jmp    b7123 <rvvdk_vmdk::stream_map::StreamMap::read_from+0xe03>
   b7434:	xor    %edx,%edx
   b7436:	lea    0x120(%rsp),%rdi
   b743e:	mov    %r14,%rsi
   b7441:	call   *0x20f879(%rip)        # 2c6cc0 <_DYNAMIC+0x610>
   b7447:	mov    0x120(%rsp),%rax
   b744f:	mov    0x128(%rsp),%rdx
   b7457:	cmp    $0x10,%rax
   b745b:	jne    b74e1 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x11c1>
   b7461:	lea    0x1c0(%rsp),%rdi
   b7469:	mov    $0x200,%esi
   b746e:	call   *0x20f84c(%rip)        # 2c6cc0 <_DYNAMIC+0x610>
   b7474:	mov    0x1c0(%rsp),%rax
   b747c:	mov    0x1c8(%rsp),%rdx
   b7484:	cmp    $0x10,%rax
   b7488:	jne    b74eb <rvvdk_vmdk::stream_map::StreamMap::read_from+0x11cb>
   b748a:	lea    0x2d0(%rsp),%rdi
   b7492:	mov    %r12,%rsi
   b7495:	call   *0x20f825(%rip)        # 2c6cc0 <_DYNAMIC+0x610>
   b749b:	mov    0x2d0(%rsp),%rax
   b74a3:	mov    0x2d8(%rsp),%rcx
   b74ab:	mov    %rcx,0x8(%rsp)
   b74b0:	cmp    $0x10,%rax
   b74b4:	jne    b7522 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1202>
   b74b6:	cmp    %rbx,0x8(%rsp)
   b74bb:	jbe    b7541 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1221>
   b74c1:	movq   $0xb,0x8(%r15)
   b74c9:	lea    -0x96e6d(%rip),%rax        # 20663 <anon.321e7b27d4937aed77e095c38e205854.42.llvm.3789170322229662815+0x7a>
   b74d0:	mov    %rax,0x10(%r15)
   b74d4:	movq   $0xe,0x18(%r15)
   b74dc:	jmp    b7123 <rvvdk_vmdk::stream_map::StreamMap::read_from+0xe03>
   b74e1:	movups 0x130(%rsp),%xmm0
   b74e9:	jmp    b74f3 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x11d3>
   b74eb:	movups 0x1d0(%rsp),%xmm0
   b74f3:	movups %xmm0,0x18(%r15)
   b74f8:	mov    %rax,0x8(%r15)
   b74fc:	mov    %rdx,0x10(%r15)
   b7500:	jmp    b7123 <rvvdk_vmdk::stream_map::StreamMap::read_from+0xe03>
   b7505:	lea    0x20261c(%rip),%rcx        # 2b9b28 <anon.321e7b27d4937aed77e095c38e205854.44.llvm.3789170322229662815+0x48>
   b750c:	jmp    b7515 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x11f5>
   b750e:	lea    0x2025fb(%rip),%rcx        # 2b9b10 <anon.321e7b27d4937aed77e095c38e205854.44.llvm.3789170322229662815+0x30>
   b7515:	xor    %edi,%edi
   b7517:	call   *0x20f613(%rip)        # 2c6b30 <_DYNAMIC+0x480>
   b751d:	jmp    b81cc <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1eac>
   b7522:	movups 0x2e0(%rsp),%xmm0
   b752a:	movups %xmm0,0x18(%r15)
   b752f:	mov    %rax,0x8(%r15)
   b7533:	mov    0x8(%rsp),%rax
   b7538:	mov    %rax,0x10(%r15)
   b753c:	jmp    b7123 <rvvdk_vmdk::stream_map::StreamMap::read_from+0xe03>
   b7541:	movq   $0x0,0x30(%rsp)
   b754a:	mov    0x20f5e7(%rip),%r14        # 2c6b38 <memset@GLIBC_2.2.5>
   b7551:	mov    $0x800,%edx
   b7556:	lea    0xdb0(%rsp),%rdi
   b755e:	xor    %esi,%esi
   b7560:	call   *%r14
   b7563:	mov    $0x800,%edx
   b7568:	lea    0x5b0(%rsp),%rdi
   b7570:	xor    %esi,%esi
   b7572:	call   *%r14
   b7575:	mov    0x298(%rsp),%rax
   b757d:	mov    %rax,0x28(%rsp)
   b7582:	mov    0x70(%rbp),%rax
   b7586:	mov    %rax,0x48(%rsp)
   b758b:	movl   $0x0,0x38(%rsp)
   b7593:	xor    %r14d,%r14d
   b7596:	mov    %r14,%r12
   b7599:	shl    $0x9,%r12
   b759d:	add    $0xfffffffffffffe00,%r12
   b75a4:	cmp    0x78(%rsp),%r14
   b75a9:	jae    b7737 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1417>
   b75af:	mov    0x18(%rsp),%rdi
   b75b4:	mov    (%rsp),%rsi
   b75b8:	mov    %r14,%rdx
   b75bb:	call   *0x20f6f7(%rip)        # 2c6cb8 <_DYNAMIC+0x608>
   b75c1:	inc    %r14
   b75c4:	add    $0x200,%r12
   b75cb:	test   %eax,%eax
   b75cd:	je     b75a4 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1284>
   b75cf:	lea    -0x1(%r14),%rax
   b75d3:	sub    $0x8,%rsp
   b75d7:	lea    0x358(%rsp),%rdi
   b75df:	lea    0x108(%rsp),%rsi
   b75e7:	mov    0x20(%rsp),%rdx
   b75ec:	mov    0x8(%rsp),%rcx
   b75f1:	mov    0x48(%rsp),%r8
   b75f6:	mov    0x58(%rsp),%r9
   b75fb:	lea    0x5b8(%rsp),%r10
   b7603:	push   %r10
   b7605:	lea    0xdc0(%rsp),%r10
   b760d:	push   %r10
   b760f:	push   %rax
   b7610:	call   b5ee0 <rvvdk_vmdk::stream_map::read_table>
   b7615:	add    $0x20,%rsp
   b7619:	cmpl   $0x10,0x350(%rsp)
   b7621:	jne    b7cca <rvvdk_vmdk::stream_map::StreamMap::read_from+0x19aa>
   b7627:	movq   $0x0,0x10(%rsp)
   b7630:	cmpq   $0x1ff,0x10(%rsp)
   b7639:	jbe    b7665 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1345>
   b763b:	jmp    b7596 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1276>
   b7640:	cmpl   $0x0,0x20(%rsp)
   b7645:	jne    b77f9 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x14d9>
   b764b:	mov    0x10(%rsp),%rcx
   b7650:	inc    %rcx
   b7653:	mov    %rcx,0x10(%rsp)
   b7658:	cmp    $0x200,%rcx
   b765f:	je     b7596 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1276>
   b7665:	mov    $0x800,%esi
   b766a:	lea    0xdb0(%rsp),%rdi
   b7672:	mov    0x10(%rsp),%rdx
   b7677:	call   *0x20f63b(%rip)        # 2c6cb8 <_DYNAMIC+0x608>
   b767d:	mov    %eax,0x20(%rsp)
   b7681:	mov    0x10(%rsp),%rax
   b7686:	add    %r12,%rax
   b7689:	cmp    0xd0(%rsp),%rax
   b7691:	jae    b7640 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1320>
   b7693:	cmpl   $0x0,0x20(%rsp)
   b7698:	je     b764b <rvvdk_vmdk::stream_map::StreamMap::read_from+0x132b>
   b769a:	cmpl   $0x1,0x20(%rsp)
   b769f:	je     b77e5 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x14c5>
   b76a5:	mov    0x20(%rsp),%esi
   b76a9:	shl    $0x9,%rsi
   b76ad:	cmp    0x28(%rsp),%rsi
   b76b2:	jb     b77e5 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x14c5>
   b76b8:	mov    $0x200,%edx
   b76bd:	lea    0x350(%rsp),%rdi
   b76c5:	call   *0x20f5f5(%rip)        # 2c6cc0 <_DYNAMIC+0x610>
   b76cb:	mov    0x350(%rsp),%rcx
   b76d3:	mov    0x358(%rsp),%rax
   b76db:	cmp    $0x10,%rcx
   b76df:	jne    b7d06 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x19e6>
   b76e5:	cmp    0x88(%rsp),%rax
   b76ed:	ja     b77e5 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x14c5>
   b76f3:	mov    0x20(%rsp),%eax
   b76f7:	cmp    0x38(%rsp),%eax
   b76fb:	jbe    b7d2c <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1a0c>
   b7701:	incq   0x10(%rsp)
   b7706:	mov    0x30(%rsp),%rcx
   b770b:	inc    %rcx
   b770e:	mov    0x20(%rsp),%eax
   b7712:	mov    %eax,0x38(%rsp)
   b7716:	mov    %rcx,0x30(%rsp)
   b771b:	cmp    0x48(%rsp),%rcx
   b7720:	jbe    b7630 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1310>
   b7726:	movq   $0xb,0x8(%r15)
   b772e:	lea    -0xa0155(%rip),%rax        # 175e0 <anon.e60e008f553044d7c15e66252433230a.30.llvm.15109012375404921823+0x80>
   b7735:	jmp    b77b5 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1495>
   b7737:	lea    0x350(%rsp),%rdi
   b773f:	mov    $0xc,%edx
   b7744:	mov    0x30(%rsp),%rsi
   b7749:	call   *0x20f579(%rip)        # 2c6cc8 <_DYNAMIC+0x618>
   b774f:	mov    0x350(%rsp),%rax
   b7757:	mov    0x358(%rsp),%rdx
   b775f:	cmp    $0x10,%rax
   b7763:	jne    b7ca3 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1983>
   b7769:	lea    0x120(%rsp),%rdi
   b7771:	mov    0x58(%rsp),%rsi
   b7776:	call   *0x20f544(%rip)        # 2c6cc0 <_DYNAMIC+0x610>
   b777c:	mov    0x120(%rsp),%rax
   b7784:	mov    0x128(%rsp),%rcx
   b778c:	mov    %rcx,0x10(%rsp)
   b7791:	cmp    $0x10,%rax
   b7795:	jne    b77c6 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x14a6>
   b7797:	mov    0x10(%rsp),%rax
   b779c:	cmp    0xa0(%rsp),%rax
   b77a4:	jbe    b7819 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x14f9>
   b77a6:	movq   $0xb,0x8(%r15)
   b77ae:	lea    -0xa04b5(%rip),%rax        # 17300 <anon.d625489d584c397ac22d75864c33158f.35.llvm.5936746164385555759+0x1a0>
   b77b5:	mov    %rax,0x10(%r15)
   b77b9:	movq   $0x10,0x18(%r15)
   b77c1:	jmp    b7123 <rvvdk_vmdk::stream_map::StreamMap::read_from+0xe03>
   b77c6:	movups 0x130(%rsp),%xmm0
   b77ce:	movups %xmm0,0x18(%r15)
   b77d3:	mov    %rax,0x8(%r15)
   b77d7:	mov    0x10(%rsp),%rax
   b77dc:	mov    %rax,0x10(%r15)
   b77e0:	jmp    b7123 <rvvdk_vmdk::stream_map::StreamMap::read_from+0xe03>
   b77e5:	movq   $0xa,0x8(%r15)
   b77ed:	lea    -0x9708d(%rip),%rax        # 20767 <anon.321e7b27d4937aed77e095c38e205854.42.llvm.3789170322229662815+0x17e>
   b77f4:	jmp    b730b <rvvdk_vmdk::stream_map::StreamMap::read_from+0xfeb>
   b77f9:	movq   $0xa,0x8(%r15)
   b7801:	lea    -0x9708d(%rip),%rax        # 2077b <anon.321e7b27d4937aed77e095c38e205854.42.llvm.3789170322229662815+0x192>
   b7808:	mov    %rax,0x10(%r15)
   b780c:	movq   $0x12,0x18(%r15)
   b7814:	jmp    b7123 <rvvdk_vmdk::stream_map::StreamMap::read_from+0xe03>
   b7819:	lea    0x350(%rsp),%rdi
   b7821:	mov    $0xc,%edx
   b7826:	mov    0x30(%rsp),%rsi
   b782b:	call   *0x20f497(%rip)        # 2c6cc8 <_DYNAMIC+0x618>
   b7831:	mov    0x350(%rsp),%rax
   b7839:	mov    0x358(%rsp),%rdx
   b7841:	cmp    $0x10,%rax
   b7845:	jne    b7ca3 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1983>
   b784b:	lea    0x120(%rsp),%rdi
   b7853:	mov    0x8(%rsp),%rsi
   b7858:	call   *0x20f462(%rip)        # 2c6cc0 <_DYNAMIC+0x610>
   b785e:	mov    0x120(%rsp),%rcx
   b7866:	mov    0x128(%rsp),%rax
   b786e:	cmp    $0x10,%rcx
   b7872:	jne    b7cb0 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1990>
   b7878:	cmp    %rbx,%rax
   b787b:	ja     b74c1 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x11a1>
   b7881:	movq   $0x0,0x120(%rsp)
   b788d:	movl   $0x0,0x128(%rsp)
   b7898:	lea    0x350(%rsp),%rdi
   b78a0:	lea    0x120(%rsp),%rdx
   b78a8:	mov    0x30(%rsp),%rsi
   b78ad:	call   b8400 <rvvdk_vmdk::stream_map::allocated>
   b78b2:	mov    0x350(%rsp),%rax
   b78ba:	mov    0x358(%rsp),%rcx
   b78c2:	mov    %rcx,0x48(%rsp)
   b78c7:	mov    0x360(%rsp),%rcx
   b78cf:	mov    %rcx,0x90(%rsp)
   b78d7:	mov    0x368(%rsp),%rcx
   b78df:	mov    %rcx,0x70(%rsp)
   b78e4:	cmp    $0x10,%rax
   b78e8:	jne    b7cdf <rvvdk_vmdk::stream_map::StreamMap::read_from+0x19bf>
   b78ee:	movq   $0x0,0x68(%rsp)
   b78f7:	movq   $0x0,0x8(%rsp)
   b7900:	mov    0x8(%rsp),%rax
   b7905:	shl    $0x9,%rax
   b7909:	mov    %rax,0x20(%rsp)
   b790e:	mov    0x8(%rsp),%r14
   b7913:	mov    0x20(%rsp),%rax
   b7918:	mov    %rax,0x58(%rsp)
   b791d:	cmp    0x78(%rsp),%r14
   b7922:	jae    b7d10 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x19f0>
   b7928:	mov    0x18(%rsp),%rdi
   b792d:	mov    (%rsp),%rsi
   b7931:	mov    %r14,%rdx
   b7934:	call   *0x20f37e(%rip)        # 2c6cb8 <_DYNAMIC+0x608>
   b793a:	mov    %eax,0xa0(%rsp)
   b7941:	lea    0x1(%r14),%rax
   b7945:	mov    %rax,0x8(%rsp)
   b794a:	mov    0x58(%rsp),%rax
   b794f:	add    $0x200,%rax
   b7955:	mov    %rax,0x20(%rsp)
   b795a:	cmpl   $0x0,0xa0(%rsp)
   b7962:	je     b790e <rvvdk_vmdk::stream_map::StreamMap::read_from+0x15ee>
   b7964:	sub    $0x8,%rsp
   b7968:	lea    0x358(%rsp),%rdi
   b7970:	lea    0x108(%rsp),%rsi
   b7978:	mov    0x20(%rsp),%rdx
   b797d:	mov    0x8(%rsp),%rcx
   b7982:	mov    0x48(%rsp),%r8
   b7987:	mov    0x58(%rsp),%r9
   b798c:	lea    0x5b8(%rsp),%rax
   b7994:	push   %rax
   b7995:	lea    0xdc0(%rsp),%rax
   b799d:	push   %rax
   b799e:	push   %r14
   b79a0:	call   b5ee0 <rvvdk_vmdk::stream_map::read_table>
   b79a5:	add    $0x20,%rsp
   b79a9:	cmpl   $0x10,0x350(%rsp)
   b79b1:	jne    b7ea7 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1b87>
   b79b7:	movq   $0x0,0xa8(%rsp)
   b79c3:	xor    %r14d,%r14d
   b79c6:	mov    0x58(%rsp),%rax
   b79cb:	mov    0xa8(%rsp),%rcx
   b79d3:	lea    (%rax,%rcx,1),%r12
   b79d7:	shl    $0x10,%r12
   b79db:	add    $0xffffffffffff0000,%r12
   b79e2:	mov    $0x201,%ebx
   b79e7:	cmp    $0x200,%r14
   b79ee:	jae    b7b66 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1846>
   b79f4:	mov    $0x800,%esi
   b79f9:	lea    0xdb0(%rsp),%rdi
   b7a01:	mov    %r14,%rdx
   b7a04:	call   *0x20f2ae(%rip)        # 2c6cb8 <_DYNAMIC+0x608>
   b7a0a:	mov    %eax,0x38(%rsp)
   b7a0e:	inc    %r14
   b7a11:	add    $0x10000,%r12
   b7a18:	dec    %rbx
   b7a1b:	cmpl   $0x0,0x38(%rsp)
   b7a20:	je     b79e7 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x16c7>
   b7a22:	mov    0x68(%rsp),%rax
   b7a27:	cmp    0x70(%rsp),%rax
   b7a2c:	je     b80bf <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1d9f>
   b7a32:	mov    0x20(%rsp),%rax
   b7a37:	mov    0xa8(%rsp),%rcx
   b7a3f:	add    %rcx,%rax
   b7a42:	sub    %rbx,%rax
   b7a45:	cmp    0xd0(%rsp),%rax
   b7a4d:	jae    b80bf <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1d9f>
   b7a53:	mov    0x38(%rsp),%eax
   b7a57:	shl    $0x9,%rax
   b7a5b:	cmp    0x28(%rsp),%rax
   b7a60:	jne    b80df <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1dbf>
   b7a66:	xorps  %xmm0,%xmm0
   b7a69:	movaps %xmm0,0x120(%rsp)
   b7a71:	mov    $0xc,%r8d
   b7a77:	lea    0x350(%rsp),%rdi
   b7a7f:	lea    0x100(%rsp),%rsi
   b7a87:	mov    0x28(%rsp),%rdx
   b7a8c:	lea    0x120(%rsp),%rcx
   b7a94:	call   b61c0 <rvvdk_vmdk::stream_map::Budget<S>::read>
   b7a99:	cmpl   $0x10,0x350(%rsp)
   b7aa1:	jne    b7ea7 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1b87>
   b7aa7:	mov    $0x10,%edx
   b7aac:	lea    0x350(%rsp),%rdi
   b7ab4:	lea    0x120(%rsp),%rsi
   b7abc:	mov    0x28(%rsp),%rcx
   b7ac1:	lea    0x250(%rsp),%r8
   b7ac9:	mov    %rbp,%r9
   b7acc:	call   *0x20f10e(%rip)        # 2c6be0 <_DYNAMIC+0x530>
   b7ad2:	mov    0x350(%rsp),%rcx
   b7ada:	mov    0x358(%rsp),%rax
   b7ae2:	mov    0x360(%rsp),%rdx
   b7aea:	mov    %rdx,0x28(%rsp)
   b7aef:	cmp    $0x5,%rcx
   b7af3:	je     b80f0 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1dd0>
   b7af9:	test   %rcx,%rcx
   b7afc:	jne    b810f <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1def>
   b7b02:	cmp    %r12,%rax
   b7b05:	jne    b8120 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1e00>
   b7b0b:	mov    0x68(%rsp),%rax
   b7b10:	cmp    0x70(%rsp),%rax
   b7b15:	jae    b81b5 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1e95>
   b7b1b:	mov    0x370(%rsp),%rax
   b7b23:	mov    0xa8(%rsp),%rdi
   b7b2b:	add    0x20(%rsp),%edi
   b7b2f:	sub    %ebx,%edi
   b7b31:	mov    0x68(%rsp),%rsi
   b7b36:	lea    (%rsi,%rsi,2),%rcx
   b7b3a:	mov    0x90(%rsp),%rdx
   b7b42:	mov    %edi,(%rdx,%rcx,4)
   b7b45:	mov    0x38(%rsp),%edi
   b7b49:	mov    %edi,0x4(%rdx,%rcx,4)
   b7b4d:	mov    %eax,0x8(%rdx,%rcx,4)
   b7b51:	inc    %rsi
   b7b54:	mov    %rsi,0x68(%rsp)
   b7b59:	mov    %r14,0xa8(%rsp)
   b7b61:	jmp    b79c6 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x16a6>
   b7b66:	cmpl   $0x2,0x98(%rsp)
   b7b6e:	jne    b7900 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x15e0>
   b7b74:	mov    0xa0(%rsp),%ebx
   b7b7b:	shl    $0x9,%rbx
   b7b7f:	lea    -0x200(%rbx),%rax
   b7b86:	cmp    %rax,0x28(%rsp)
   b7b8b:	jne    b8160 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1e40>
   b7b91:	xorps  %xmm0,%xmm0
   b7b94:	movaps %xmm0,0x2d0(%rsp)
   b7b9c:	mov    $0x10,%r8d
   b7ba2:	lea    0x350(%rsp),%rdi
   b7baa:	lea    0x100(%rsp),%rsi
   b7bb2:	mov    0x28(%rsp),%rdx
   b7bb7:	lea    0x2d0(%rsp),%rcx
   b7bbf:	call   b61c0 <rvvdk_vmdk::stream_map::Budget<S>::read>
   b7bc4:	cmpl   $0x10,0x350(%rsp)
   b7bcc:	jne    b7ea7 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1b87>
   b7bd2:	mov    $0x10,%edx
   b7bd7:	lea    0x350(%rsp),%rdi
   b7bdf:	lea    0x2d0(%rsp),%rsi
   b7be7:	mov    0x28(%rsp),%rcx
   b7bec:	lea    0x250(%rsp),%r8
   b7bf4:	mov    %rbp,%r9
   b7bf7:	call   *0x20efe3(%rip)        # 2c6be0 <_DYNAMIC+0x530>
   b7bfd:	mov    0x350(%rsp),%rax
   b7c05:	lea    0x358(%rsp),%rcx
   b7c0d:	movups (%rcx),%xmm0
   b7c10:	movups 0x10(%rcx),%xmm1
   b7c14:	movaps %xmm0,0x1c0(%rsp)
   b7c1c:	movaps %xmm1,0x1d0(%rsp)
   b7c24:	cmp    $0x5,%rax
   b7c28:	je     b8180 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1e60>
   b7c2e:	movaps 0x1c0(%rsp),%xmm0
   b7c36:	movaps 0x1d0(%rsp),%xmm1
   b7c3e:	lea    0x128(%rsp),%rcx
   b7c46:	movups %xmm1,0x10(%rcx)
   b7c4a:	movups %xmm0,(%rcx)
   b7c4d:	mov    %rax,0x120(%rsp)
   b7c55:	mov    %rbx,0x358(%rsp)
   b7c5d:	movq   $0x800,0x360(%rsp)
   b7c69:	movq   $0x1,0x350(%rsp)
   b7c75:	lea    0x120(%rsp),%rdi
   b7c7d:	lea    0x350(%rsp),%rsi
   b7c85:	call   b8640 <<rvvdk_vmdk::stream::StreamMarker as core::cmp::PartialEq>::eq>
   b7c8a:	test   %al,%al
   b7c8c:	je     b8195 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1e75>
   b7c92:	add    $0x800,%rbx
   b7c99:	mov    %rbx,0x28(%rsp)
   b7c9e:	jmp    b7900 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x15e0>
   b7ca3:	movups 0x360(%rsp),%xmm0
   b7cab:	jmp    b74f3 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x11d3>
   b7cb0:	movups 0x130(%rsp),%xmm0
   b7cb8:	movups %xmm0,0x18(%r15)
   b7cbd:	mov    %rcx,0x8(%r15)
   b7cc1:	mov    %rax,0x10(%r15)
   b7cc5:	jmp    b7123 <rvvdk_vmdk::stream_map::StreamMap::read_from+0xe03>
   b7cca:	movups 0x350(%rsp),%xmm0
   b7cd2:	movups 0x360(%rsp),%xmm1
   b7cda:	jmp    b7119 <rvvdk_vmdk::stream_map::StreamMap::read_from+0xdf9>
   b7cdf:	mov    0x48(%rsp),%rcx
   b7ce4:	mov    %rcx,0x10(%r15)
   b7ce8:	mov    0x90(%rsp),%rcx
   b7cf0:	mov    %rcx,0x18(%r15)
   b7cf4:	mov    0x70(%rsp),%rcx
   b7cf9:	mov    %rcx,0x20(%r15)
   b7cfd:	mov    %rax,0x8(%r15)
   b7d01:	jmp    b7123 <rvvdk_vmdk::stream_map::StreamMap::read_from+0xe03>
   b7d06:	movups 0x360(%rsp),%xmm0
   b7d0e:	jmp    b7cb8 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1998>
   b7d10:	cmpl   $0x2,0x98(%rsp)
   b7d18:	je     b7d4c <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1a2c>
   b7d1a:	mov    0x88(%rsp),%rax
   b7d22:	mov    %rax,0x80(%rsp)
   b7d2a:	jmp    b7d58 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1a38>
   b7d2c:	movq   $0xa,0x8(%r15)
   b7d34:	lea    -0x975ed(%rip),%rax        # 2074e <anon.321e7b27d4937aed77e095c38e205854.42.llvm.3789170322229662815+0x165>
   b7d3b:	mov    %rax,0x10(%r15)
   b7d3f:	movq   $0x19,0x18(%r15)
   b7d47:	jmp    b7123 <rvvdk_vmdk::stream_map::StreamMap::read_from+0xe03>
   b7d4c:	addq   $0xfffffffffffffe00,0x80(%rsp)
   b7d58:	mov    0x28(%rsp),%rax
   b7d5d:	cmp    0x80(%rsp),%rax
   b7d65:	jne    b7e78 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1b58>
   b7d6b:	mov    0x68(%rsp),%rax
   b7d70:	cmp    0x70(%rsp),%rax
   b7d75:	jne    b7e78 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1b58>
   b7d7b:	lea    0x350(%rsp),%r14
   b7d83:	mov    $0x200,%edx
   b7d88:	mov    %r14,%rdi
   b7d8b:	xor    %esi,%esi
   b7d8d:	call   *0x20eda5(%rip)        # 2c6b38 <memset@GLIBC_2.2.5>
   b7d93:	lea    0x120(%rsp),%rdi
   b7d9b:	lea    0x100(%rsp),%rsi
   b7da3:	mov    $0x200,%r8d
   b7da9:	xor    %edx,%edx
   b7dab:	mov    %r14,%rcx
   b7dae:	call   b61c0 <rvvdk_vmdk::stream_map::Budget<S>::read>
   b7db3:	cmpl   $0x10,0x120(%rsp)
   b7dbb:	jne    b7e95 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1b75>
   b7dc1:	mov    0x50(%rbp),%rax
   b7dc5:	mov    %rax,0x320(%rsp)
   b7dcd:	movups 0x40(%rbp),%xmm0
   b7dd1:	movaps %xmm0,0x310(%rsp)
   b7dd9:	movups 0x0(%rbp),%xmm0
   b7ddd:	movups 0x10(%rbp),%xmm1
   b7de1:	movups 0x20(%rbp),%xmm2
   b7de5:	movups 0x30(%rbp),%xmm3
   b7de9:	movaps %xmm3,0x300(%rsp)
   b7df1:	movaps %xmm2,0x2f0(%rsp)
   b7df9:	movaps %xmm1,0x2e0(%rsp)
   b7e01:	movaps %xmm0,0x2d0(%rsp)
   b7e09:	mov    0x88(%rsp),%rcx
   b7e11:	lea    0x120(%rsp),%rdi
   b7e19:	lea    0x350(%rsp),%rsi
   b7e21:	lea    0x2d0(%rsp),%r8
   b7e29:	mov    $0x200,%edx
   b7e2e:	call   *0x20ed9c(%rip)        # 2c6bd0 <_DYNAMIC+0x520>
   b7e34:	mov    0x120(%rsp),%rax
   b7e3c:	movups 0x128(%rsp),%xmm0
   b7e44:	movaps %xmm0,0x220(%rsp)
   b7e4c:	movups 0x138(%rsp),%xmm0
   b7e54:	movaps %xmm0,0x230(%rsp)
   b7e5c:	cmp    $0x3,%rax
   b7e60:	jne    b7ef9 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1bd9>
   b7e66:	movaps 0x220(%rsp),%xmm0
   b7e6e:	movaps 0x230(%rsp),%xmm1
   b7e76:	jmp    b7eb7 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1b97>
   b7e78:	movq   $0xa,0x8(%r15)
   b7e80:	lea    -0x977ce(%rip),%rax        # 206b9 <anon.321e7b27d4937aed77e095c38e205854.42.llvm.3789170322229662815+0xd0>
   b7e87:	mov    %rax,0x10(%r15)
   b7e8b:	movq   $0x26,0x18(%r15)
   b7e93:	jmp    b7ec1 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1ba1>
   b7e95:	movups 0x120(%rsp),%xmm0
   b7e9d:	movups 0x130(%rsp),%xmm1
   b7ea5:	jmp    b7eb7 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1b97>
   b7ea7:	movups 0x350(%rsp),%xmm0
   b7eaf:	movups 0x360(%rsp),%xmm1
   b7eb7:	movups %xmm1,0x18(%r15)
   b7ebc:	movups %xmm0,0x8(%r15)
   b7ec1:	movq   $0x3,(%r15)
   b7ec8:	cmpq   $0x0,0x48(%rsp)
   b7ece:	je     b712a <rvvdk_vmdk::stream_map::StreamMap::read_from+0xe0a>
   b7ed4:	mov    0x48(%rsp),%rax
   b7ed9:	shl    $0x2,%rax
   b7edd:	lea    (%rax,%rax,2),%rsi
   b7ee1:	mov    $0x4,%edx
   b7ee6:	mov    0x90(%rsp),%rdi
   b7eee:	call   *0x20ea0c(%rip)        # 2c6900 <_DYNAMIC+0x250>
   b7ef4:	jmp    b712a <rvvdk_vmdk::stream_map::StreamMap::read_from+0xe0a>
   b7ef9:	mov    0x178(%rsp),%rcx
   b7f01:	mov    %rcx,0x218(%rsp)
   b7f09:	movups 0x148(%rsp),%xmm0
   b7f11:	movups 0x158(%rsp),%xmm1
   b7f19:	movups 0x168(%rsp),%xmm2
   b7f21:	movups %xmm2,0x208(%rsp)
   b7f29:	movups %xmm1,0x1f8(%rsp)
   b7f31:	movups %xmm0,0x1e8(%rsp)
   b7f39:	movaps 0x220(%rsp),%xmm0
   b7f41:	movaps 0x230(%rsp),%xmm1
   b7f49:	movups %xmm0,0x1c8(%rsp)
   b7f51:	movups %xmm1,0x1d8(%rsp)
   b7f59:	mov    %rax,0x1c0(%rsp)
   b7f61:	lea    0x1c0(%rsp),%rdi
   b7f69:	lea    0x250(%rsp),%rsi
   b7f71:	call   b85f0 <<rvvdk_vmdk::stream::StreamHeader as core::cmp::PartialEq>::eq>
   b7f76:	test   %al,%al
   b7f78:	je     b80ab <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1d8b>
   b7f7e:	mov    0x100(%rsp),%rax
   b7f86:	mov    (%rax),%rcx
   b7f89:	mov    0x10(%rcx),%rcx
   b7f8d:	mov    %rcx,0x8(%rax)
   b7f91:	cmp    %rcx,0x88(%rsp)
   b7f99:	jne    b8140 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1e20>
   b7f9f:	mov    0x110(%rsp),%rax
   b7fa7:	movaps 0x330(%rsp),%xmm0
   b7faf:	movaps 0x340(%rsp),%xmm1
   b7fb7:	movups %xmm1,0x18(%r15)
   b7fbc:	movups %xmm0,0x8(%r15)
   b7fc1:	mov    0x1b0(%rsp),%rcx
   b7fc9:	mov    %rcx,0x58(%r15)
   b7fcd:	movaps 0x180(%rsp),%xmm0
   b7fd5:	movaps 0x190(%rsp),%xmm1
   b7fdd:	movaps 0x1a0(%rsp),%xmm2
   b7fe5:	movups %xmm2,0x48(%r15)
   b7fea:	movups %xmm1,0x38(%r15)
   b7fef:	movups %xmm0,0x28(%r15)
   b7ff4:	mov    0x48(%rsp),%rcx
   b7ff9:	mov    %rcx,0x60(%r15)
   b7ffd:	mov    0x90(%rsp),%rcx
   b8005:	mov    %rcx,0x68(%r15)
   b8009:	mov    0x70(%rsp),%rcx
   b800e:	mov    %rcx,0x70(%r15)
   b8012:	mov    0x98(%rsp),%rcx
   b801a:	mov    %rcx,(%r15)
   b801d:	mov    %rax,0x78(%r15)
   b8021:	mov    %r13,0x80(%r15)
   b8028:	mov    0x30(%rsp),%rax
   b802d:	mov    %rax,0x88(%r15)
   b8034:	mov    0x10(%rsp),%rax
   b8039:	mov    %rax,0x90(%r15)
   b8040:	mov    0xe4(%rsp),%eax
   b8047:	mov    %eax,0x98(%r15)
   b804e:	mov    0xb0(%rsp),%rsi
   b8056:	test   %rsi,%rsi
   b8059:	je     b8072 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1d52>
   b805b:	mov    0xb8(%rsp),%rdi
   b8063:	shl    $0x4,%rsi
   b8067:	mov    $0x8,%edx
   b806c:	call   *0x20e88e(%rip)        # 2c6900 <_DYNAMIC+0x250>
   b8072:	cmpq   $0x0,0x60(%rsp)
   b8078:	je     b808f <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1d6f>
   b807a:	mov    $0x1,%edx
   b807f:	mov    0x40(%rsp),%rdi
   b8084:	mov    0x60(%rsp),%rsi
   b8089:	call   *0x20e871(%rip)        # 2c6900 <_DYNAMIC+0x250>
   b808f:	mov    (%rsp),%rsi
   b8093:	test   %rsi,%rsi
   b8096:	mov    0x18(%rsp),%rdi
   b809b:	je     b6b85 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x865>
   b80a1:	mov    $0x1,%edx
   b80a6:	jmp    b6b7f <rvvdk_vmdk::stream_map::StreamMap::read_from+0x85f>
   b80ab:	movq   $0xa,0x8(%r15)
   b80b3:	lea    -0x97a13(%rip),%rax        # 206a7 <anon.321e7b27d4937aed77e095c38e205854.42.llvm.3789170322229662815+0xbe>
   b80ba:	jmp    b814f <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1e2f>
   b80bf:	movq   $0xa,0x8(%r15)
   b80c7:	lea    -0x97993(%rip),%rax        # 2073b <anon.321e7b27d4937aed77e095c38e205854.42.llvm.3789170322229662815+0x152>
   b80ce:	mov    %rax,0x10(%r15)
   b80d2:	movq   $0x13,0x18(%r15)
   b80da:	jmp    b7ec1 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1ba1>
   b80df:	movq   $0xa,0x8(%r15)
   b80e7:	lea    -0x979c8(%rip),%rax        # 20726 <anon.321e7b27d4937aed77e095c38e205854.42.llvm.3789170322229662815+0x13d>
   b80ee:	jmp    b816f <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1e4f>
   b80f0:	movups 0x368(%rsp),%xmm0
   b80f8:	mov    %rax,0x8(%r15)
   b80fc:	mov    0x28(%rsp),%rax
   b8101:	mov    %rax,0x10(%r15)
   b8105:	movups %xmm0,0x18(%r15)
   b810a:	jmp    b7ec1 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1ba1>
   b810f:	movq   $0xa,0x8(%r15)
   b8117:	lea    -0x97a0d(%rip),%rax        # 20711 <anon.321e7b27d4937aed77e095c38e205854.42.llvm.3789170322229662815+0x128>
   b811e:	jmp    b816f <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1e4f>
   b8120:	movq   $0xa,0x8(%r15)
   b8128:	lea    -0x97a2f(%rip),%rax        # 20700 <anon.321e7b27d4937aed77e095c38e205854.42.llvm.3789170322229662815+0x117>
   b812f:	mov    %rax,0x10(%r15)
   b8133:	movq   $0x11,0x18(%r15)
   b813b:	jmp    b7ec1 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1ba1>
   b8140:	movq   $0xa,0x8(%r15)
   b8148:	lea    -0x97aba(%rip),%rax        # 20695 <anon.321e7b27d4937aed77e095c38e205854.42.llvm.3789170322229662815+0xac>
   b814f:	mov    %rax,0x10(%r15)
   b8153:	movq   $0x12,0x18(%r15)
   b815b:	jmp    b7ec1 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1ba1>
   b8160:	movq   $0xa,0x8(%r15)
   b8168:	lea    -0x97a84(%rip),%rax        # 206eb <anon.321e7b27d4937aed77e095c38e205854.42.llvm.3789170322229662815+0x102>
   b816f:	mov    %rax,0x10(%r15)
   b8173:	movq   $0x15,0x18(%r15)
   b817b:	jmp    b7ec1 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1ba1>
   b8180:	movaps 0x1c0(%rsp),%xmm0
   b8188:	movaps 0x1d0(%rsp),%xmm1
   b8190:	jmp    b7eb7 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1b97>
   b8195:	movq   $0xa,0x8(%r15)
   b819d:	lea    -0x97ac5(%rip),%rax        # 206df <anon.321e7b27d4937aed77e095c38e205854.42.llvm.3789170322229662815+0xf6>
   b81a4:	mov    %rax,0x10(%r15)
   b81a8:	movq   $0xc,0x18(%r15)
   b81b0:	jmp    b7ec1 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1ba1>
   b81b5:	lea    0x20193c(%rip),%rdx        # 2b9af8 <anon.321e7b27d4937aed77e095c38e205854.44.llvm.3789170322229662815+0x18>
   b81bc:	mov    0x68(%rsp),%rdi
   b81c1:	mov    0x70(%rsp),%rsi
   b81c6:	call   *0x20e7cc(%rip)        # 2c6998 <_DYNAMIC+0x2e8>
   b81cc:	ud2
   b81ce:	jmp    b81d6 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1eb6>
   b81d0:	jmp    b81d6 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1eb6>
   b81d2:	jmp    b81d6 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1eb6>
   b81d4:	jmp    b81d6 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1eb6>
   b81d6:	mov    %rax,%rbx
   b81d9:	cmpq   $0x0,0x48(%rsp)
   b81df:	je     b821a <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1efa>
   b81e1:	mov    0x48(%rsp),%rax
   b81e6:	shl    $0x2,%rax
   b81ea:	lea    (%rax,%rax,2),%rsi
   b81ee:	mov    $0x4,%edx
   b81f3:	mov    0x90(%rsp),%rdi
   b81fb:	call   *0x20e6ff(%rip)        # 2c6900 <_DYNAMIC+0x250>
   b8201:	jmp    b821a <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1efa>
   b8203:	jmp    b8217 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1ef7>
   b8205:	jmp    b8217 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1ef7>
   b8207:	jmp    b8217 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1ef7>
   b8209:	jmp    b8217 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1ef7>
   b820b:	jmp    b8217 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1ef7>
   b820d:	jmp    b8217 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1ef7>
   b820f:	jmp    b8217 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1ef7>
   b8211:	jmp    b8217 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1ef7>
   b8213:	jmp    b8217 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1ef7>
   b8215:	jmp    b8217 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1ef7>
   b8217:	mov    %rax,%rbx
   b821a:	mov    0xb0(%rsp),%rsi
   b8222:	test   %rsi,%rsi
   b8225:	je     b8243 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1f23>
   b8227:	mov    0xb8(%rsp),%rdi
   b822f:	shl    $0x4,%rsi
   b8233:	mov    $0x8,%edx
   b8238:	call   *0x20e6c2(%rip)        # 2c6900 <_DYNAMIC+0x250>
   b823e:	jmp    b8243 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1f23>
   b8240:	mov    %rax,%rbx
   b8243:	cmpq   $0x0,0x60(%rsp)
   b8249:	je     b8260 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1f40>
   b824b:	mov    $0x1,%edx
   b8250:	mov    0x40(%rsp),%rdi
   b8255:	mov    0x60(%rsp),%rsi
   b825a:	call   *0x20e6a0(%rip)        # 2c6900 <_DYNAMIC+0x250>
   b8260:	cmpq   $0x0,(%rsp)
   b8265:	je     b827b <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1f5b>
   b8267:	mov    $0x1,%edx
   b826c:	mov    0x18(%rsp),%rdi
   b8271:	mov    (%rsp),%rsi
   b8275:	call   *0x20e685(%rip)        # 2c6900 <_DYNAMIC+0x250>
   b827b:	mov    %rbx,%rdi
   b827e:	call   2b82e0 <_Unwind_Resume@plt>
