
target/r510p/reference/descriptor-before:     file format elf64-x86-64


Disassembly of section .text:

00000000000c1480 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits>:
   c1480:	push   %rbp
   c1481:	push   %r15
   c1483:	push   %r14
   c1485:	push   %r13
   c1487:	push   %r12
   c1489:	push   %rbx
   c148a:	sub    $0x1f8,%rsp
   c1491:	mov    %rdi,%rbx
   c1494:	movabs $0x8000000000000000,%r15
   c149e:	cmp    (%rcx),%rdx
   c14a1:	jbe    c14ba <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x3a>
   c14a3:	lea    -0xaa41a(%rip),%r12        # 17090 <anon.83d0f48a2d0b5ff7b6e24226a84ebc25.25.llvm.569205349683904225+0x60>
   c14aa:	mov    $0x10,%ebp
   c14af:	xor    %r13d,%r13d
   c14b2:	xor    %r14d,%r14d
   c14b5:	jmp    c254f <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x10cf>
   c14ba:	mov    %rcx,%r14
   c14bd:	lea    0x120(%rsp),%rdi
   c14c5:	call   *0x1f69bd(%rip)        # 2b7e88 <_DYNAMIC+0x7e0>
   c14cb:	cmpl   $0x1,0x120(%rsp)
   c14d3:	jne    c14e3 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x63>
   c14d5:	mov    $0x1,%r13d
   c14db:	xor    %r14d,%r14d
   c14de:	jmp    c254f <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x10cf>
   c14e3:	mov    0x128(%rsp),%rax
   c14eb:	mov    0x130(%rsp),%rcx
   c14f3:	movq   $0x0,0x80(%rsp)
   c14ff:	movq   $0x8,0x88(%rsp)
   c150b:	movq   $0x0,0x90(%rsp)
   c1517:	movq   $0x0,0x68(%rsp)
   c1520:	movq   $0x8,0x70(%rsp)
   c1529:	movq   $0x0,0x78(%rsp)
   c1532:	xorps  %xmm0,%xmm0
   c1535:	movaps %xmm0,0xb0(%rsp)
   c153d:	mov    %rcx,0xc0(%rsp)
   c1545:	lea    0xc8(%rsp),%rsi
   c154d:	mov    %rax,0xc8(%rsp)
   c1555:	mov    %rcx,0xd0(%rsp)
   c155d:	movq   $0x0,0xd8(%rsp)
   c1569:	mov    %rcx,0xe0(%rsp)
   c1571:	movabs $0xa0000000a,%rax
   c157b:	mov    %rax,0xe8(%rsp)
   c1583:	movb   $0x1,0xf0(%rsp)
   c158b:	movw   $0x1,0xf8(%rsp)
   c1595:	mov    0x8(%r14),%rax
   c1599:	mov    %rax,0x1f0(%rsp)
   c15a1:	mov    0x10(%r14),%rax
   c15a5:	mov    %rax,0x1e8(%rsp)
   c15ad:	mov    0x20(%r14),%rax
   c15b1:	mov    %rax,0x1e0(%rsp)
   c15b9:	mov    0x18(%r14),%rax
   c15bd:	mov    %rax,0x1d8(%rsp)
   c15c5:	mov    $0x5,%bpl
   c15c8:	lea    0x120(%rsp),%rdi
   c15d0:	lea    -0xa08e1(%rip),%r13        # 20cf6 <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.12047894789963178251+0x287>
   c15d7:	movl   $0x0,0x54(%rsp)
   c15df:	movl   $0x0,0x50(%rsp)
   c15e7:	movq   $0x0,0xa0(%rsp)
   c15f3:	movq   $0x0,0x118(%rsp)
   c15ff:	movl   $0x0,0x18(%rsp)
   c1607:	movq   $0x0,0x60(%rsp)
   c1610:	mov    0xc8(%rsp),%r12
   c1618:	call   c1270 <<core::str::pattern::CharSearcher as core::str::pattern::Searcher>::next_match>
   c161d:	cmpl   $0x1,0x120(%rsp)
   c1625:	jne    c164f <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1cf>
   c1627:	mov    0x128(%rsp),%rax
   c162f:	mov    0x130(%rsp),%rcx
   c1637:	mov    0xb8(%rsp),%rdx
   c163f:	sub    %rdx,%rax
   c1642:	add    %rdx,%r12
   c1645:	mov    %rcx,0xb8(%rsp)
   c164d:	jmp    c1693 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x213>
   c164f:	cmpb   $0x0,0xf9(%rsp)
   c1657:	jne    c2675 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x11f5>
   c165d:	movb   $0x1,0xf9(%rsp)
   c1665:	mov    0xb8(%rsp),%r12
   c166d:	mov    0xc0(%rsp),%rax
   c1675:	sub    %r12,%rax
   c1678:	setne  %cl
   c167b:	or     0xf8(%rsp),%cl
   c1682:	cmp    $0x1,%cl
   c1685:	jne    c2675 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x11f5>
   c168b:	add    0xc8(%rsp),%r12
   c1693:	mov    0xb0(%rsp),%r14
   c169b:	inc    %r14
   c169e:	mov    %r14,0xb0(%rsp)
   c16a6:	cmp    0x1f0(%rsp),%rax
   c16ae:	ja     c2577 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x10f7>
   c16b4:	xor    %edx,%edx
   c16b6:	test   %rax,%rax
   c16b9:	je     c16c7 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x247>
   c16bb:	cmpb   $0xd,-0x1(%r12,%rax,1)
   c16c1:	sete   %dl
   c16c4:	neg    %rdx
   c16c7:	add    %rax,%rdx
   c16ca:	lea    (%r12,%rdx,1),%rax
   c16ce:	mov    %r12,%rcx
   c16d1:	jmp    c1709 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x289>
   c16d3:	data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   c16e0:	inc    %rcx
   c16e3:	lea    -0x7f(%rsi),%edi
   c16e6:	xor    %r8d,%r8d
   c16e9:	cmp    $0x21,%edi
   c16ec:	setb   %r8b
   c16f0:	xor    %edi,%edi
   c16f2:	cmp    $0x9,%esi
   c16f5:	setne  %dil
   c16f9:	cmp    $0x20,%esi
   c16fc:	cmovb  %edi,%r8d
   c1700:	test   %r8b,%r8b
   c1703:	jne    c24db <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x105b>
   c1709:	cmp    %rax,%rcx
   c170c:	je     c178a <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x30a>
   c170e:	movzbl (%rcx),%esi
   c1711:	test   %sil,%sil
   c1714:	jns    c16e0 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x260>
   c1716:	mov    %esi,%edi
   c1718:	and    $0x1f,%edi
   c171b:	movzbl 0x1(%rcx),%r9d
   c1720:	and    $0x3f,%r9d
   c1724:	cmp    $0xdf,%sil
   c1728:	jbe    c1767 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x2e7>
   c172a:	movzbl 0x2(%rcx),%r8d
   c172f:	shl    $0x6,%r9d
   c1733:	and    $0x3f,%r8d
   c1737:	or     %r9d,%r8d
   c173a:	cmp    $0xf0,%sil
   c173e:	jb     c1778 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x2f8>
   c1740:	movzbl 0x3(%rcx),%esi
   c1744:	and    $0x7,%edi
   c1747:	shl    $0x12,%edi
   c174a:	shl    $0x6,%r8d
   c174e:	and    $0x3f,%esi
   c1751:	or     %r8d,%esi
   c1754:	or     %edi,%esi
   c1756:	cmp    $0x110000,%esi
   c175c:	je     c178a <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x30a>
   c175e:	add    $0x4,%rcx
   c1762:	jmp    c16e3 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x263>
   c1767:	add    $0x2,%rcx
   c176b:	shl    $0x6,%edi
   c176e:	or     %r9d,%edi
   c1771:	mov    %edi,%esi
   c1773:	jmp    c16e3 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x263>
   c1778:	add    $0x3,%rcx
   c177c:	shl    $0xc,%edi
   c177f:	or     %edi,%r8d
   c1782:	mov    %r8d,%esi
   c1785:	jmp    c16e3 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x263>
   c178a:	lea    0x120(%rsp),%rdi
   c1792:	mov    %r12,%rsi
   c1795:	mov    %r14,%rcx
   c1798:	call   c2d60 <rvvdk_vmdk::descriptor::tokens>
   c179d:	mov    %bpl,0x7(%rsp)
   c17a2:	mov    0x120(%rsp),%rax
   c17aa:	mov    0x128(%rsp),%rdi
   c17b2:	mov    0x130(%rsp),%r9
   c17ba:	mov    0x138(%rsp),%rbp
   c17c2:	mov    0x140(%rsp),%r13
   c17ca:	cmp    $0x3,%rax
   c17ce:	je     c2591 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1111>
   c17d4:	mov    0x1b0(%rsp),%r12
   c17dc:	cmp    $0x7,%r12
   c17e0:	jae    c27e7 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1367>
   c17e6:	mov    0x148(%rsp),%r15
   c17ee:	mov    0x150(%rsp),%rcx
   c17f6:	mov    %rcx,0x58(%rsp)
   c17fb:	mov    0x158(%rsp),%rdx
   c1803:	mov    0x160(%rsp),%r8
   c180b:	mov    0x168(%rsp),%rcx
   c1813:	mov    %rcx,0x48(%rsp)
   c1818:	mov    0x170(%rsp),%r11
   c1820:	mov    0x178(%rsp),%r10
   c1828:	mov    0x180(%rsp),%rcx
   c1830:	mov    %rcx,0x1d0(%rsp)
   c1838:	mov    0x188(%rsp),%rcx
   c1840:	mov    %rcx,0x1c0(%rsp)
   c1848:	mov    0x190(%rsp),%rcx
   c1850:	mov    %rcx,0x1c8(%rsp)
   c1858:	cmp    $0x3,%r12
   c185c:	je     c18a3 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x423>
   c185e:	test   %r12,%r12
   c1861:	lea    0xc8(%rsp),%rsi
   c1869:	jne    c18b3 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x433>
   c186b:	movabs $0x8000000000000000,%r15
   c1875:	lea    -0xa0b86(%rip),%r13        # 20cf6 <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.12047894789963178251+0x287>
   c187c:	lea    -0xa0d97(%rip),%r12        # 20aec <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.12047894789963178251+0x7d>
   c1883:	movzbl 0x7(%rsp),%ebp
   c1888:	cmpb   $0x0,0xf9(%rsp)
   c1890:	lea    0x120(%rsp),%rdi
   c1898:	je     c1610 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x190>
   c189e:	jmp    c2686 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1206>
   c18a3:	mov    %rbp,%rcx
   c18a6:	xor    $0x2,%rcx
   c18aa:	or     %rax,%rcx
   c18ad:	je     c1d17 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x897>
   c18b3:	cmpl   $0x2,0x18(%rsp)
   c18b8:	je     c259f <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x111f>
   c18be:	mov    0x90(%rsp),%rcx
   c18c6:	mov    %rcx,0x18(%rsp)
   c18cb:	cmp    0x1e8(%rsp),%rcx
   c18d3:	jae    c25b6 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1136>
   c18d9:	test   %rax,%rax
   c18dc:	jne    c25ca <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x114a>
   c18e2:	cmp    $0x3,%r12
   c18e6:	jb     c25ca <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x114a>
   c18ec:	test   %rbp,%rbp
   c18ef:	jne    c25ca <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x114a>
   c18f5:	cmpq   $0x0,0x58(%rsp)
   c18fb:	jne    c25ca <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x114a>
   c1901:	mov    %r11,0x1b8(%rsp)
   c1909:	mov    %r10,0x58(%rsp)
   c190e:	mov    %rbx,0x38(%rsp)
   c1913:	mov    %r14,%rbx
   c1916:	mov    %r8,0x28(%rsp)
   c191b:	mov    %rdx,0x30(%rsp)
   c1920:	mov    $0x2,%ecx
   c1925:	mov    %r9,%rsi
   c1928:	lea    -0xa0d0c(%rip),%rdx        # 20c23 <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.12047894789963178251+0x1b4>
   c192f:	mov    %rdi,%rbp
   c1932:	mov    %r9,%r14
   c1935:	call   c2ad0 <rvvdk_vmdk::descriptor::eq>
   c193a:	mov    %eax,%edx
   c193c:	test   %al,%al
   c193e:	jne    c1963 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x4e3>
   c1940:	mov    $0x6,%ecx
   c1945:	mov    %rbp,%rdi
   c1948:	mov    %r14,%rsi
   c194b:	mov    %edx,%ebp
   c194d:	lea    -0xa0d2f(%rip),%rdx        # 20c25 <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.12047894789963178251+0x1b6>
   c1954:	call   c2ad0 <rvvdk_vmdk::descriptor::eq>
   c1959:	mov    %ebp,%edx
   c195b:	test   %al,%al
   c195d:	je     c25ee <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x116e>
   c1963:	test   %r15,%r15
   c1966:	je     c25e1 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1161>
   c196c:	add    $0xfffffffffffffffd,%r12
   c1970:	xor    $0x1,%dl
   c1973:	xor    %eax,%eax
   c1975:	mov    %rbx,%r14
   c1978:	nopl   0x0(%rax,%rax,1)
   c1980:	cmp    %rax,%r15
   c1983:	je     c199b <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x51b>
   c1985:	movzbl 0x0(%r13,%rax,1),%ecx
   c198b:	add    $0xc6,%cl
   c198e:	inc    %rax
   c1991:	cmp    $0xf6,%cl
   c1994:	jae    c1980 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x500>
   c1996:	jmp    c24e3 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1063>
   c199b:	movzbl 0x0(%r13),%eax
   c19a0:	cmp    $0x1,%r15
   c19a4:	mov    0x38(%rsp),%rbx
   c19a9:	jne    c19bd <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x53d>
   c19ab:	cmp    $0x2b,%eax
   c19ae:	je     c24f5 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1075>
   c19b4:	cmp    $0x2d,%eax
   c19b7:	je     c24f5 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1075>
   c19bd:	mov    %dl,0x8(%rsp)
   c19c1:	xor    %edx,%edx
   c19c3:	cmp    $0x2b,%eax
   c19c6:	sete   %dl
   c19c9:	mov    %r15,%rcx
   c19cc:	sub    %rdx,%rcx
   c19cf:	add    %rdx,%r13
   c19d2:	mov    %rdx,%rax
   c19d5:	neg    %rax
   c19d8:	cmp    $0x11,%rcx
   c19dc:	jae    c1b10 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x690>
   c19e2:	test   %rcx,%rcx
   c19e5:	je     c260d <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x118d>
   c19eb:	add    %rax,%r15
   c19ee:	neg    %r15
   c19f1:	xor    %ecx,%ecx
   c19f3:	xor    %eax,%eax
   c19f5:	data16 cs nopw 0x0(%rax,%rax,1)
   c1a00:	movzbl 0x0(%r13,%rcx,1),%edx
   c1a06:	add    $0xffffffd0,%edx
   c1a09:	cmp    $0x9,%edx
   c1a0c:	ja     c24f5 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1075>
   c1a12:	lea    (%rax,%rax,4),%rax
   c1a16:	mov    %edx,%edx
   c1a18:	lea    (%rdx,%rax,2),%rax
   c1a1c:	inc    %rcx
   c1a1f:	mov    %r15,%rdx
   c1a22:	add    %rcx,%rdx
   c1a25:	jne    c1a00 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x580>
   c1a27:	mov    %rax,%rcx
   c1a2a:	shr    $0x37,%rcx
   c1a2e:	mov    $0x7,%r13d
   c1a34:	jne    c28f2 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1472>
   c1a3a:	test   %rax,%rax
   c1a3d:	mov    0x30(%rsp),%rbp
   c1a42:	je     c264f <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x11cf>
   c1a48:	mov    %rax,0x40(%rsp)
   c1a4d:	shl    $0x9,%rax
   c1a51:	mov    %rax,0x10(%rsp)
   c1a56:	mov    $0x4,%ecx
   c1a5b:	mov    %rbp,%rdi
   c1a5e:	mov    0x28(%rsp),%r15
   c1a63:	mov    %r15,%rsi
   c1a66:	lea    -0xa2cfd(%rip),%rdx        # 1ed70 <anon.d625489d584c397ac22d75864c33158f.21.llvm.5936746164385555759+0x30>
   c1a6d:	call   c2ad0 <rvvdk_vmdk::descriptor::eq>
   c1a72:	test   %al,%al
   c1a74:	je     c1b58 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x6d8>
   c1a7a:	mov    $0x21,%ebp
   c1a7f:	cmp    $0x2,%r12
   c1a83:	jne    c272d <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x12ad>
   c1a89:	cmpq   $0x1,0x48(%rsp)
   c1a8f:	mov    0x40(%rsp),%rcx
   c1a94:	mov    0x58(%rsp),%r10
   c1a99:	mov    0x1b8(%rsp),%r11
   c1aa1:	jne    c272d <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x12ad>
   c1aa7:	cmpq   $0x0,0x1d0(%rsp)
   c1ab0:	jne    c272d <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x12ad>
   c1ab6:	mov    $0xe,%ebp
   c1abb:	test   %r10,%r10
   c1abe:	je     c2756 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x12d6>
   c1ac4:	cmp    0x1e0(%rsp),%r10
   c1acc:	ja     c277f <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x12ff>
   c1ad2:	mov    0x1c8(%rsp),%r12
   c1ada:	test   %r12,%r12
   c1add:	je     c2587 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1107>
   c1ae3:	xor    %eax,%eax
   c1ae5:	movzbl 0x7(%rsp),%edi
   c1aea:	mov    0x1c0(%rsp),%r15
   c1af2:	cmp    %rax,%r12
   c1af5:	je     c1c7e <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x7fe>
   c1afb:	movzbl (%r15,%rax,1),%edx
   c1b00:	add    $0xc6,%dl
   c1b03:	inc    %rax
   c1b06:	cmp    $0xf6,%dl
   c1b09:	jae    c1af2 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x672>
   c1b0b:	jmp    c2587 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1107>
   c1b10:	add    %rax,%r15
   c1b13:	neg    %r15
   c1b16:	xor    %ecx,%ecx
   c1b18:	xor    %eax,%eax
   c1b1a:	mov    %r15,%rdx
   c1b1d:	add    %rcx,%rdx
   c1b20:	je     c1a27 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x5a7>
   c1b26:	mov    $0xa,%edx
   c1b2b:	mul    %rdx
   c1b2e:	jo     c24f5 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1075>
   c1b34:	movzbl 0x0(%r13,%rcx,1),%esi
   c1b3a:	add    $0xffffffd0,%esi
   c1b3d:	add    %rsi,%rax
   c1b40:	setb   %dl
   c1b43:	cmp    $0x9,%esi
   c1b46:	ja     c24f5 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1075>
   c1b4c:	inc    %rcx
   c1b4f:	test   %dl,%dl
   c1b51:	je     c1b1a <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x69a>
   c1b53:	jmp    c24f5 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1075>
   c1b58:	mov    $0x4,%ecx
   c1b5d:	mov    %rbp,%rdi
   c1b60:	mov    %r15,%rsi
   c1b63:	lea    -0xa2e6e(%rip),%rdx        # 1ecfc <anon.83d0f48a2d0b5ff7b6e24226a84ebc25.18.llvm.569205349683904225+0x14>
   c1b6a:	call   c2ad0 <rvvdk_vmdk::descriptor::eq>
   c1b6f:	test   %al,%al
   c1b71:	movzbl 0x7(%rsp),%eax
   c1b76:	je     c273f <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x12bf>
   c1b7c:	test   %r12,%r12
   c1b7f:	jne    c2768 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x12e8>
   c1b85:	mov    $0x1,%r15d
   c1b8b:	cmp    $0x2,%al
   c1b8d:	mov    0x110(%rsp),%rcx
   c1b95:	mov    0x108(%rsp),%rdx
   c1b9d:	jne    c2624 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x11a4>
   c1ba3:	mov    0x10(%rsp),%r12
   c1ba8:	add    0x60(%rsp),%r12
   c1bad:	lea    -0xa0ebe(%rip),%rax        # 20cf6 <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.12047894789963178251+0x287>
   c1bb4:	jb     c2500 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1080>
   c1bba:	mov    %rdx,0x108(%rsp)
   c1bc2:	mov    %rcx,0x110(%rsp)
   c1bca:	mov    0x18(%rsp),%r14
   c1bcf:	cmp    0x80(%rsp),%r14
   c1bd7:	mov    %rax,%r13
   c1bda:	movzbl 0x7(%rsp),%ebp
   c1bdf:	jne    c1bef <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x76f>
   c1be1:	lea    0x80(%rsp),%rdi
   c1be9:	call   *0x1f63b9(%rip)        # 2b7fa8 <_DYNAMIC+0x900>
   c1bef:	mov    0x88(%rsp),%rax
   c1bf7:	imul   $0x38,%r14,%rcx
   c1bfb:	mov    %r15,(%rax,%rcx,1)
   c1bff:	mov    0x110(%rsp),%rdx
   c1c07:	mov    %rdx,0x8(%rax,%rcx,1)
   c1c0c:	mov    0x108(%rsp),%rdx
   c1c14:	mov    %rdx,0x10(%rax,%rcx,1)
   c1c19:	mov    0x98(%rsp),%rdx
   c1c21:	mov    %rdx,0x18(%rax,%rcx,1)
   c1c26:	mov    0x60(%rsp),%rdx
   c1c2b:	mov    %rdx,0x20(%rax,%rcx,1)
   c1c30:	mov    0x10(%rsp),%rdx
   c1c35:	mov    %rdx,0x28(%rax,%rcx,1)
   c1c3a:	movzbl 0x8(%rsp),%edx
   c1c3f:	mov    %dl,0x30(%rax,%rcx,1)
   c1c43:	inc    %r14
   c1c46:	mov    %r14,0x90(%rsp)
   c1c4e:	movl   $0x1,0x18(%rsp)
   c1c56:	cmpb   $0x0,0xf9(%rsp)
   c1c5e:	mov    %r12,0x60(%rsp)
   c1c63:	lea    0xc8(%rsp),%rsi
   c1c6b:	lea    0x120(%rsp),%rdi
   c1c73:	je     c1610 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x190>
   c1c79:	jmp    c2670 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x11f0>
   c1c7e:	movzbl (%r15),%eax
   c1c82:	cmp    $0x1,%r12
   c1c86:	jne    c1c9a <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x81a>
   c1c88:	cmp    $0x2b,%eax
   c1c8b:	je     c2587 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1107>
   c1c91:	cmp    $0x2d,%eax
   c1c94:	je     c2587 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1107>
   c1c9a:	xor    %esi,%esi
   c1c9c:	cmp    $0x2b,%eax
   c1c9f:	sete   %sil
   c1ca3:	mov    %r12,%rdx
   c1ca6:	sub    %rsi,%rdx
   c1ca9:	add    %rsi,%r15
   c1cac:	mov    %rsi,%rax
   c1caf:	neg    %rax
   c1cb2:	cmp    $0x11,%rdx
   c1cb6:	jae    c1d48 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x8c8>
   c1cbc:	test   %rdx,%rdx
   c1cbf:	je     c20d1 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0xc51>
   c1cc5:	add    %rax,%r12
   c1cc8:	neg    %r12
   c1ccb:	xor    %edx,%edx
   c1ccd:	xor    %eax,%eax
   c1ccf:	mov    0x10(%rsp),%r8
   c1cd4:	mov    0x20(%rsp),%r9
   c1cd9:	movzbl (%r15,%rdx,1),%esi
   c1cde:	add    $0xffffffd0,%esi
   c1ce1:	cmp    $0x9,%esi
   c1ce4:	ja     c2647 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x11c7>
   c1cea:	lea    (%rax,%rax,4),%rax
   c1cee:	mov    %esi,%esi
   c1cf0:	lea    (%rsi,%rax,2),%rax
   c1cf4:	inc    %rdx
   c1cf7:	mov    %r12,%rsi
   c1cfa:	add    %rdx,%rsi
   c1cfd:	jne    c1cd9 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x859>
   c1cff:	movabs $0x7fffffffffffff,%rdx
   c1d09:	cmp    %rdx,%rax
   c1d0c:	jbe    c20d8 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0xc58>
   c1d12:	jmp    c29f7 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1577>
   c1d17:	cmp    $0x4,%r9
   c1d1b:	mov    %rdx,0x30(%rsp)
   c1d20:	mov    %r8,0x28(%rsp)
   c1d25:	mov    %r9,0x8(%rsp)
   c1d2a:	jbe    c1d94 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x914>
   c1d2c:	cmpb   $0xc0,0x4(%rdi)
   c1d30:	movabs $0x8000000000000000,%r15
   c1d3a:	lea    -0xa1255(%rip),%r12        # 20aec <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.12047894789963178251+0x7d>
   c1d41:	jge    c1dab <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x92b>
   c1d43:	jmp    c1f80 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0xb00>
   c1d48:	add    %rax,%r12
   c1d4b:	neg    %r12
   c1d4e:	xor    %esi,%esi
   c1d50:	xor    %eax,%eax
   c1d52:	mov    %r12,%rdx
   c1d55:	add    %rsi,%rdx
   c1d58:	je     c242c <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0xfac>
   c1d5e:	mov    $0xa,%ecx
   c1d63:	mul    %rcx
   c1d66:	jo     c2587 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1107>
   c1d6c:	movzbl (%r15,%rsi,1),%ecx
   c1d71:	add    $0xffffffd0,%ecx
   c1d74:	add    %rcx,%rax
   c1d77:	setb   %dl
   c1d7a:	cmp    $0x9,%ecx
   c1d7d:	mov    0x40(%rsp),%rcx
   c1d82:	ja     c2587 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1107>
   c1d88:	inc    %rsi
   c1d8b:	test   %dl,%dl
   c1d8d:	je     c1d52 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x8d2>
   c1d8f:	jmp    c2587 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1107>
   c1d94:	movabs $0x8000000000000000,%r15
   c1d9e:	lea    -0xa12b9(%rip),%r12        # 20aec <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.12047894789963178251+0x7d>
   c1da5:	jne    c1f80 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0xb00>
   c1dab:	movzbl (%rdi),%eax
   c1dae:	lea    -0x41(%rax),%ecx
   c1db1:	cmp    $0x1a,%cl
   c1db4:	setb   %cl
   c1db7:	shl    $0x5,%cl
   c1dba:	or     %al,%cl
   c1dbc:	cmp    $0x64,%cl
   c1dbf:	jne    c1f80 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0xb00>
   c1dc5:	movzbl 0x1(%rdi),%eax
   c1dc9:	lea    -0x41(%rax),%ecx
   c1dcc:	cmp    $0x1a,%cl
   c1dcf:	setb   %cl
   c1dd2:	shl    $0x5,%cl
   c1dd5:	or     %al,%cl
   c1dd7:	cmp    $0x64,%cl
   c1dda:	jne    c1f80 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0xb00>
   c1de0:	movzbl 0x2(%rdi),%eax
   c1de4:	lea    -0x41(%rax),%ecx
   c1de7:	cmp    $0x1a,%cl
   c1dea:	setb   %cl
   c1ded:	shl    $0x5,%cl
   c1df0:	or     %al,%cl
   c1df2:	cmp    $0x62,%cl
   c1df5:	jne    c1f80 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0xb00>
   c1dfb:	movzbl 0x3(%rdi),%eax
   c1dff:	lea    -0x41(%rax),%ecx
   c1e02:	cmp    $0x1a,%cl
   c1e05:	setb   %cl
   c1e08:	shl    $0x5,%cl
   c1e0b:	or     %al,%cl
   c1e0d:	cmp    $0x2e,%cl
   c1e10:	jne    c1f80 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0xb00>
   c1e16:	mov    %rbx,0x38(%rsp)
   c1e1b:	mov    %r14,0x48(%rsp)
   c1e20:	mov    $0x2,%r13d
   c1e26:	cmpl   $0x0,0x18(%rsp)
   c1e2b:	je     c2993 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1513>
   c1e31:	mov    %rdi,%r15
   c1e34:	mov    $0xf,%ecx
   c1e39:	mov    0x8(%rsp),%rsi
   c1e3e:	lea    -0xa12b1(%rip),%rdx        # 20b94 <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.12047894789963178251+0x125>
   c1e45:	call   c2ad0 <rvvdk_vmdk::descriptor::eq>
   c1e4a:	test   %al,%al
   c1e4c:	jne    c1f29 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0xaa9>
   c1e52:	mov    $0x16,%ecx
   c1e57:	mov    %r15,%rdi
   c1e5a:	mov    0x8(%rsp),%rsi
   c1e5f:	lea    -0xa12c3(%rip),%rdx        # 20ba3 <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.12047894789963178251+0x134>
   c1e66:	call   c2ad0 <rvvdk_vmdk::descriptor::eq>
   c1e6b:	test   %al,%al
   c1e6d:	jne    c1f29 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0xaa9>
   c1e73:	mov    $0x12,%ecx
   c1e78:	mov    %r15,%rdi
   c1e7b:	mov    0x8(%rsp),%rsi
   c1e80:	lea    -0xa12ce(%rip),%rdx        # 20bb9 <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.12047894789963178251+0x14a>
   c1e87:	call   c2ad0 <rvvdk_vmdk::descriptor::eq>
   c1e8c:	test   %al,%al
   c1e8e:	jne    c1f29 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0xaa9>
   c1e94:	mov    $0x14,%ecx
   c1e99:	mov    %r15,%rdi
   c1e9c:	mov    0x8(%rsp),%rsi
   c1ea1:	lea    -0xa12dd(%rip),%rdx        # 20bcb <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.12047894789963178251+0x15c>
   c1ea8:	call   c2ad0 <rvvdk_vmdk::descriptor::eq>
   c1ead:	test   %al,%al
   c1eaf:	jne    c1f29 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0xaa9>
   c1eb1:	mov    $0x14,%ecx
   c1eb6:	mov    %r15,%rdi
   c1eb9:	mov    0x8(%rsp),%rsi
   c1ebe:	lea    -0xa12e6(%rip),%rdx        # 20bdf <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.12047894789963178251+0x170>
   c1ec5:	call   c2ad0 <rvvdk_vmdk::descriptor::eq>
   c1eca:	test   %al,%al
   c1ecc:	jne    c1f29 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0xaa9>
   c1ece:	mov    $0x10,%ecx
   c1ed3:	mov    %r15,%rdi
   c1ed6:	mov    0x8(%rsp),%rsi
   c1edb:	lea    -0xaabb2(%rip),%rdx        # 17330 <anon.786f9ca1aa6bf68944dde0bec4ae4f7b.2.llvm.4745231054059156856+0x70>
   c1ee2:	call   c2ad0 <rvvdk_vmdk::descriptor::eq>
   c1ee7:	test   %al,%al
   c1ee9:	jne    c1f29 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0xaa9>
   c1eeb:	mov    $0x8,%ecx
   c1ef0:	mov    %r15,%rdi
   c1ef3:	mov    0x8(%rsp),%rsi
   c1ef8:	lea    -0xaa7d7(%rip),%rdx        # 17728 <anon.d625489d584c397ac22d75864c33158f.39.llvm.5936746164385555759+0x20>
   c1eff:	call   c2ad0 <rvvdk_vmdk::descriptor::eq>
   c1f04:	test   %al,%al
   c1f06:	jne    c1f29 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0xaa9>
   c1f08:	mov    $0x11,%ecx
   c1f0d:	mov    %r15,%rdi
   c1f10:	mov    0x8(%rsp),%rsi
   c1f15:	lea    -0xa1329(%rip),%rdx        # 20bf3 <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.12047894789963178251+0x184>
   c1f1c:	call   c2ad0 <rvvdk_vmdk::descriptor::eq>
   c1f21:	test   %al,%al
   c1f23:	je     c29ff <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x157f>
   c1f29:	cmpq   $0x1,0x58(%rsp)
   c1f2f:	jne    c29a4 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1524>
   c1f35:	mov    0x70(%rsp),%rsi
   c1f3a:	mov    0x78(%rsp),%rbp
   c1f3f:	mov    %rbp,%rbx
   c1f42:	shl    $0x5,%rbp
   c1f46:	mov    %rsi,%r14
   c1f49:	test   %rbp,%rbp
   c1f4c:	mov    0x8(%rsp),%rcx
   c1f51:	je     c244e <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0xfce>
   c1f57:	mov    %r15,%rdx
   c1f5a:	lea    0x20(%rsi),%r12
   c1f5e:	mov    (%rsi),%rdi
   c1f61:	mov    0x8(%rsi),%rsi
   c1f65:	call   c2ad0 <rvvdk_vmdk::descriptor::eq>
   c1f6a:	add    $0xffffffffffffffe0,%rbp
   c1f6e:	mov    $0x3,%r13d
   c1f74:	mov    %r12,%rsi
   c1f77:	test   %al,%al
   c1f79:	je     c1f49 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0xac9>
   c1f7b:	jmp    c284c <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x13cc>
   c1f80:	mov    $0x2,%r13d
   c1f86:	mov    $0x14,%ebp
   c1f8b:	cmpl   $0x0,0x18(%rsp)
   c1f90:	jne    c2800 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1380>
   c1f96:	mov    0x58(%rsp),%rax
   c1f9b:	test   %rax,%rax
   c1f9e:	mov    %rdi,0x40(%rsp)
   c1fa3:	je     c2081 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0xc01>
   c1fa9:	cmp    $0x1,%rax
   c1fad:	jne    c280c <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x138c>
   c1fb3:	mov    $0xa,%ecx
   c1fb8:	mov    %r9,%rsi
   c1fbb:	lea    -0xa14b1(%rip),%rdx        # 20b11 <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.12047894789963178251+0xa2>
   c1fc2:	call   c2ad0 <rvvdk_vmdk::descriptor::eq>
   c1fc7:	test   %al,%al
   c1fc9:	je     c21aa <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0xd2a>
   c1fcf:	mov    $0xe,%ecx
   c1fd4:	mov    0x30(%rsp),%rdi
   c1fd9:	mov    0x28(%rsp),%rsi
   c1fde:	lea    -0xa12e5(%rip),%rdx        # 20d00 <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.12047894789963178251+0x291>
   c1fe5:	call   c2ad0 <rvvdk_vmdk::descriptor::eq>
   c1fea:	movl   $0x0,0x18(%rsp)
   c1ff2:	mov    $0x0,%ebp
   c1ff7:	test   %al,%al
   c1ff9:	lea    -0xa130a(%rip),%r13        # 20cf6 <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.12047894789963178251+0x287>
   c2000:	jne    c2069 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0xbe9>
   c2002:	mov    $0x12,%ecx
   c2007:	mov    0x30(%rsp),%rdi
   c200c:	mov    0x28(%rsp),%rsi
   c2011:	lea    -0xa130a(%rip),%rdx        # 20d0e <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.12047894789963178251+0x29f>
   c2018:	call   c2ad0 <rvvdk_vmdk::descriptor::eq>
   c201d:	mov    $0x1,%bpl
   c2020:	test   %al,%al
   c2022:	jne    c2069 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0xbe9>
   c2024:	mov    $0x10,%ecx
   c2029:	mov    0x30(%rsp),%rdi
   c202e:	mov    0x28(%rsp),%rsi
   c2033:	lea    -0xab33a(%rip),%rdx        # 16d00 <anon.e9ff7b7b9ea76053b3f0064765461615.59.llvm.11370392988746934037+0xb0>
   c203a:	call   c2ad0 <rvvdk_vmdk::descriptor::eq>
   c203f:	test   %al,%al
   c2041:	jne    c2069 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0xbe9>
   c2043:	mov    $0x6,%ecx
   c2048:	mov    0x30(%rsp),%rdi
   c204d:	mov    0x28(%rsp),%rsi
   c2052:	lea    -0xa1339(%rip),%rdx        # 20d20 <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.12047894789963178251+0x2b1>
   c2059:	call   c2ad0 <rvvdk_vmdk::descriptor::eq>
   c205e:	mov    $0x2,%bpl
   c2061:	test   %al,%al
   c2063:	je     c29b5 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1535>
   c2069:	cmpb   $0x5,0x7(%rsp)
   c206e:	lea    0xc8(%rsp),%rsi
   c2076:	je     c1888 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x408>
   c207c:	jmp    c2841 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x13c1>
   c2081:	mov    $0x7,%ecx
   c2086:	mov    %r9,%rsi
   c2089:	mov    %r12,%rdx
   c208c:	call   c2ad0 <rvvdk_vmdk::descriptor::eq>
   c2091:	test   %al,%al
   c2093:	je     c2119 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0xc99>
   c2099:	mov    $0x6,%r13d
   c209f:	mov    0x28(%rsp),%rsi
   c20a4:	test   %rsi,%rsi
   c20a7:	je     c2666 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x11e6>
   c20ad:	xor    %eax,%eax
   c20af:	mov    0x30(%rsp),%rdx
   c20b4:	cmp    %rax,%rsi
   c20b7:	je     c2329 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0xea9>
   c20bd:	movzbl (%rdx,%rax,1),%ecx
   c20c1:	add    $0xc6,%cl
   c20c4:	inc    %rax
   c20c7:	cmp    $0xf6,%cl
   c20ca:	jae    c20b4 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0xc34>
   c20cc:	jmp    c2666 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x11e6>
   c20d1:	xor    %eax,%eax
   c20d3:	mov    0x10(%rsp),%r8
   c20d8:	mov    %rax,%rsi
   c20db:	shl    $0x9,%rsi
   c20df:	mov    %rsi,%rdx
   c20e2:	add    %r8,%rdx
   c20e5:	jb     c24e1 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1061>
   c20eb:	cmp    $0x2,%dil
   c20ef:	jae    c2231 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0xdb1>
   c20f5:	test   %rax,%rax
   c20f8:	jne    c2630 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x11b0>
   c20fe:	test   %dil,%dil
   c2101:	je     c224d <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0xdcd>
   c2107:	cmp    $0x400000,%rcx
   c210e:	jbe    c2259 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0xdd9>
   c2114:	jmp    c2919 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1499>
   c2119:	mov    $0x3,%ecx
   c211e:	mov    0x40(%rsp),%rdi
   c2123:	mov    0x8(%rsp),%rsi
   c2128:	lea    -0xa1621(%rip),%rdx        # 20b0e <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.12047894789963178251+0x9f>
   c212f:	call   c2ad0 <rvvdk_vmdk::descriptor::eq>
   c2134:	test   %al,%al
   c2136:	je     c228e <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0xe0e>
   c213c:	lea    0x120(%rsp),%rdi
   c2144:	mov    0x30(%rsp),%rsi
   c2149:	mov    0x28(%rsp),%rdx
   c214e:	mov    %r14,%rcx
   c2151:	call   c2c20 <rvvdk_vmdk::descriptor::cid>
   c2156:	mov    0x120(%rsp),%r13
   c215e:	cmp    $0x9,%r13
   c2162:	movzbl 0x7(%rsp),%ebp
   c2167:	jne    c28fc <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x147c>
   c216d:	cmpl   $0x1,0x50(%rsp)
   c2172:	lea    0xc8(%rsp),%rsi
   c217a:	je     c2841 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x13c1>
   c2180:	mov    0x128(%rsp),%eax
   c2187:	mov    %eax,0xac(%rsp)
   c218e:	movl   $0x1,0x50(%rsp)
   c2196:	movl   $0x0,0x18(%rsp)
   c219e:	lea    -0xa14af(%rip),%r13        # 20cf6 <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.12047894789963178251+0x287>
   c21a5:	jmp    c1888 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x408>
   c21aa:	mov    $0x8,%ecx
   c21af:	mov    0x40(%rsp),%rdi
   c21b4:	mov    0x8(%rsp),%rsi
   c21b9:	lea    -0xaac28(%rip),%rdx        # 17598 <anon.786f9ca1aa6bf68944dde0bec4ae4f7b.2.llvm.4745231054059156856+0x2d8>
   c21c0:	call   c2ad0 <rvvdk_vmdk::descriptor::eq>
   c21c5:	test   %al,%al
   c21c7:	mov    0x40(%rsp),%rdi
   c21cc:	mov    0x8(%rsp),%r9
   c21d1:	je     c280c <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x138c>
   c21d7:	mov    $0x5,%ecx
   c21dc:	mov    0x30(%rsp),%rdi
   c21e1:	mov    0x28(%rsp),%rsi
   c21e6:	lea    -0xa16b9(%rip),%rdx        # 20b34 <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.12047894789963178251+0xc5>
   c21ed:	call   c2ad0 <rvvdk_vmdk::descriptor::eq>
   c21f2:	mov    %eax,%ecx
   c21f4:	not    %cl
   c21f6:	or     0x118(%rsp),%cl
   c21fd:	test   $0x1,%cl
   c2200:	jne    c2930 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x14b0>
   c2206:	mov    $0x1,%al
   c2208:	mov    %rax,0x118(%rsp)
   c2210:	movl   $0x0,0x18(%rsp)
   c2218:	lea    -0xa1529(%rip),%r13        # 20cf6 <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.12047894789963178251+0x287>
   c221f:	movzbl 0x7(%rsp),%ebp
   c2224:	lea    0xc8(%rsp),%rsi
   c222c:	jmp    c1888 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x408>
   c2231:	mov    %rsi,0x20(%rsp)
   c2236:	movzbl %dil,%eax
   c223a:	cmp    $0x2,%eax
   c223d:	jne    c278e <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x130e>
   c2243:	xor    %r15d,%r15d
   c2246:	mov    0x20(%rsp),%rcx
   c224b:	jmp    c2267 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0xde7>
   c224d:	cmpq   $0x0,0x18(%rsp)
   c2253:	jne    c294e <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x14ce>
   c2259:	movq   $0x0,0x20(%rsp)
   c2262:	xor    %r15d,%r15d
   c2265:	xor    %ecx,%ecx
   c2267:	mov    %r11,%rdx
   c226a:	mov    %r10,0x98(%rsp)
   c2272:	mov    0x10(%rsp),%r12
   c2277:	add    0x60(%rsp),%r12
   c227c:	lea    -0xa158d(%rip),%rax        # 20cf6 <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.12047894789963178251+0x287>
   c2283:	jae    c1bba <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x73a>
   c2289:	jmp    c2500 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1080>
   c228e:	mov    $0x9,%ecx
   c2293:	mov    0x40(%rsp),%rdi
   c2298:	mov    0x8(%rsp),%rsi
   c229d:	lea    -0xa17b1(%rip),%rdx        # 20af3 <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.12047894789963178251+0x84>
   c22a4:	call   c2ad0 <rvvdk_vmdk::descriptor::eq>
   c22a9:	test   %al,%al
   c22ab:	mov    0x40(%rsp),%rdi
   c22b0:	mov    0x8(%rsp),%r9
   c22b5:	je     c280c <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x138c>
   c22bb:	lea    0x120(%rsp),%rdi
   c22c3:	mov    0x30(%rsp),%rsi
   c22c8:	mov    0x28(%rsp),%rdx
   c22cd:	mov    %r14,%rcx
   c22d0:	call   c2c20 <rvvdk_vmdk::descriptor::cid>
   c22d5:	mov    0x120(%rsp),%r13
   c22dd:	cmp    $0x9,%r13
   c22e1:	jne    c28fc <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x147c>
   c22e7:	cmpl   $0xffffffff,0x128(%rsp)
   c22ef:	lea    -0xa1600(%rip),%r13        # 20cf6 <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.12047894789963178251+0x287>
   c22f6:	movzbl 0x7(%rsp),%ebp
   c22fb:	lea    0xc8(%rsp),%rsi
   c2303:	jne    c2965 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x14e5>
   c2309:	cmpl   $0x1,0x54(%rsp)
   c230e:	je     c2841 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x13c1>
   c2314:	movl   $0x1,0x54(%rsp)
   c231c:	movl   $0x0,0x18(%rsp)
   c2324:	jmp    c1888 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x408>
   c2329:	movzbl (%rdx),%eax
   c232c:	cmp    $0x1,%rsi
   c2330:	jne    c2344 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0xec4>
   c2332:	cmp    $0x2b,%eax
   c2335:	je     c2666 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x11e6>
   c233b:	cmp    $0x2d,%eax
   c233e:	je     c2666 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x11e6>
   c2344:	xor    %ecx,%ecx
   c2346:	cmp    $0x2b,%eax
   c2349:	sete   %cl
   c234c:	mov    %rsi,%rax
   c234f:	sub    %rcx,%rax
   c2352:	add    %rcx,%rdx
   c2355:	neg    %rcx
   c2358:	cmp    $0x11,%rax
   c235c:	jae    c23e6 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0xf66>
   c2362:	test   %rax,%rax
   c2365:	je     c297c <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x14fc>
   c236b:	mov    %rdx,%rdi
   c236e:	add    0x28(%rsp),%rcx
   c2373:	neg    %rcx
   c2376:	xor    %edx,%edx
   c2378:	xor    %eax,%eax
   c237a:	movzbl (%rdi,%rdx,1),%esi
   c237e:	add    $0xffffffd0,%esi
   c2381:	cmp    $0x9,%esi
   c2384:	ja     c2666 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x11e6>
   c238a:	lea    (%rax,%rax,4),%rax
   c238e:	mov    %esi,%esi
   c2390:	lea    (%rsi,%rax,2),%rax
   c2394:	inc    %rdx
   c2397:	mov    %rcx,%rsi
   c239a:	add    %rdx,%rsi
   c239d:	jne    c237a <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0xefa>
   c239f:	cmp    $0x1,%rax
   c23a3:	jne    c297c <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x14fc>
   c23a9:	testb  $0x1,0xa0(%rsp)
   c23b1:	jne    c2841 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x13c1>
   c23b7:	mov    $0x1,%al
   c23b9:	mov    %rax,0xa0(%rsp)
   c23c1:	movl   $0x0,0x18(%rsp)
   c23c9:	movabs $0x8000000000000000,%r15
   c23d3:	lea    -0xa16e4(%rip),%r13        # 20cf6 <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.12047894789963178251+0x287>
   c23da:	lea    -0xa18f5(%rip),%r12        # 20aec <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.12047894789963178251+0x7d>
   c23e1:	jmp    c221f <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0xd9f>
   c23e6:	mov    %rdx,%r8
   c23e9:	add    %rsi,%rcx
   c23ec:	neg    %rcx
   c23ef:	xor    %esi,%esi
   c23f1:	xor    %eax,%eax
   c23f3:	mov    %rcx,%rdx
   c23f6:	add    %rsi,%rdx
   c23f9:	je     c239f <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0xf1f>
   c23fb:	mov    $0xa,%edx
   c2400:	mul    %rdx
   c2403:	jo     c2666 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x11e6>
   c2409:	movzbl (%r8,%rsi,1),%edi
   c240e:	add    $0xffffffd0,%edi
   c2411:	add    %rdi,%rax
   c2414:	setb   %dl
   c2417:	cmp    $0x9,%edi
   c241a:	ja     c2666 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x11e6>
   c2420:	inc    %rsi
   c2423:	test   %dl,%dl
   c2425:	je     c23f3 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0xf73>
   c2427:	jmp    c2666 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x11e6>
   c242c:	mov    0x10(%rsp),%r8
   c2431:	mov    0x20(%rsp),%r9
   c2436:	movabs $0x7fffffffffffff,%rdx
   c2440:	cmp    %rdx,%rax
   c2443:	jbe    c20d8 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0xc58>
   c2449:	jmp    c29f7 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1577>
   c244e:	cmp    0x1d8(%rsp),%rbx
   c2456:	jae    c29cc <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x154c>
   c245c:	cmp    0x68(%rsp),%rbx
   c2461:	jne    c2473 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0xff3>
   c2463:	lea    0x68(%rsp),%rdi
   c2468:	call   *0x1f5b42(%rip)        # 2b7fb0 <_DYNAMIC+0x908>
   c246e:	mov    0x70(%rsp),%r14
   c2473:	mov    %rbx,%rax
   c2476:	shl    $0x5,%rax
   c247a:	mov    %r15,(%r14,%rax,1)
   c247e:	mov    0x8(%rsp),%rcx
   c2483:	mov    %rcx,0x8(%r14,%rax,1)
   c2488:	mov    0x30(%rsp),%rcx
   c248d:	mov    %rcx,0x10(%r14,%rax,1)
   c2492:	mov    0x28(%rsp),%rcx
   c2497:	mov    %rcx,0x18(%r14,%rax,1)
   c249c:	inc    %rbx
   c249f:	mov    %rbx,0x78(%rsp)
   c24a4:	movl   $0x2,0x18(%rsp)
   c24ac:	movabs $0x8000000000000000,%r15
   c24b6:	lea    -0xa17c7(%rip),%r13        # 20cf6 <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.12047894789963178251+0x287>
   c24bd:	lea    -0xa19d8(%rip),%r12        # 20aec <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.12047894789963178251+0x7d>
   c24c4:	movzbl 0x7(%rsp),%ebp
   c24c9:	lea    0xc8(%rsp),%rsi
   c24d1:	mov    0x38(%rsp),%rbx
   c24d6:	jmp    c1888 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x408>
   c24db:	mov    $0x1,%r13d
   c24e1:	jmp    c2500 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1080>
   c24e3:	mov    0x10(%rsp),%rax
   c24e8:	mov    $0x6,%r13d
   c24ee:	mov    0x38(%rsp),%rbx
   c24f3:	jmp    c2500 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1080>
   c24f5:	mov    0x10(%rsp),%rax
   c24fa:	mov    $0x6,%r13d
   c2500:	mov    %rax,%r12
   c2503:	movabs $0x8000000000000000,%r15
   c250d:	mov    0x68(%rsp),%rsi
   c2512:	test   %rsi,%rsi
   c2515:	je     c252b <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x10ab>
   c2517:	mov    0x70(%rsp),%rdi
   c251c:	shl    $0x5,%rsi
   c2520:	mov    $0x8,%edx
   c2525:	call   *0x1f53cd(%rip)        # 2b78f8 <_DYNAMIC+0x250>
   c252b:	mov    0x80(%rsp),%rax
   c2533:	test   %rax,%rax
   c2536:	je     c254f <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x10cf>
   c2538:	mov    0x88(%rsp),%rdi
   c2540:	imul   $0x38,%rax,%rsi
   c2544:	mov    $0x8,%edx
   c2549:	call   *0x1f53a9(%rip)        # 2b78f8 <_DYNAMIC+0x250>
   c254f:	mov    %r13,0x8(%rbx)
   c2553:	mov    %r12,0x10(%rbx)
   c2557:	mov    %rbp,0x18(%rbx)
   c255b:	mov    %r14,0x20(%rbx)
   c255f:	mov    %r15,(%rbx)
   c2562:	mov    %rbx,%rax
   c2565:	add    $0x1f8,%rsp
   c256c:	pop    %rbx
   c256d:	pop    %r12
   c256f:	pop    %r13
   c2571:	pop    %r14
   c2573:	pop    %r15
   c2575:	pop    %rbp
   c2576:	ret
   c2577:	mov    $0xa,%ebp
   c257c:	mov    %r13,%rax
   c257f:	xor    %r13d,%r13d
   c2582:	jmp    c2500 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1080>
   c2587:	mov    0x20(%rsp),%rax
   c258c:	jmp    c24fa <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x107a>
   c2591:	mov    %r9,%rax
   c2594:	mov    %r13,%r14
   c2597:	mov    %rdi,%r13
   c259a:	jmp    c2500 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1080>
   c259f:	mov    $0x10,%ebp
   c25a4:	lea    -0xaba7b(%rip),%rax        # 16b30 <__abi_tag+0x16834>
   c25ab:	mov    $0x2,%r13d
   c25b1:	jmp    c2500 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1080>
   c25b6:	mov    $0x7,%ebp
   c25bb:	lea    -0xa1aa7(%rip),%rax        # 20b1b <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.12047894789963178251+0xac>
   c25c2:	xor    %r13d,%r13d
   c25c5:	jmp    c2500 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1080>
   c25ca:	lea    -0xa18e8(%rip),%rax        # 20ce9 <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.12047894789963178251+0x27a>
   c25d1:	mov    $0xd,%ebp
   c25d6:	mov    $0x2,%r13d
   c25dc:	jmp    c2500 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1080>
   c25e1:	mov    0x10(%rsp),%rax
   c25e6:	mov    $0x6,%r13d
   c25ec:	jmp    c2600 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1180>
   c25ee:	mov    $0x5,%r13d
   c25f4:	lea    -0xa19d0(%rip),%rax        # 20c2b <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.12047894789963178251+0x1bc>
   c25fb:	mov    $0xd,%ebp
   c2600:	mov    %rbx,%r14
   c2603:	mov    0x38(%rsp),%rbx
   c2608:	jmp    c2500 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1080>
   c260d:	mov    $0x8,%r13d
   c2613:	mov    $0xc,%ebp
   c2618:	lea    -0xa19e7(%rip),%rax        # 20c38 <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.12047894789963178251+0x1c9>
   c261f:	jmp    c2500 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1080>
   c2624:	movzbl %al,%eax
   c2627:	cmp    $0x2,%eax
   c262a:	jae    c278e <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x130e>
   c2630:	mov    $0x28,%ebp
   c2635:	lea    -0xa19c6(%rip),%rax        # 20c76 <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.12047894789963178251+0x207>
   c263c:	mov    $0x8,%r13d
   c2642:	jmp    c2500 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1080>
   c2647:	mov    %r9,%rax
   c264a:	jmp    c24fa <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x107a>
   c264f:	mov    $0xc,%ebp
   c2654:	lea    -0xa1a23(%rip),%rax        # 20c38 <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.12047894789963178251+0x1c9>
   c265b:	mov    $0x8,%r13d
   c2661:	jmp    c2500 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1080>
   c2666:	mov    $0x1,%eax
   c266b:	jmp    c24e1 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1061>
   c2670:	mov    %r12,0x60(%rsp)
   c2675:	movabs $0x8000000000000000,%r15
   c267f:	lea    -0xa1b9a(%rip),%r12        # 20aec <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.12047894789963178251+0x7d>
   c2686:	mov    %ebp,%ecx
   c2688:	mov    $0x4,%r13d
   c268e:	mov    $0x7,%ebp
   c2693:	cmpb   $0x1,0xa0(%rsp)
   c269b:	jne    c26c4 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1244>
   c269d:	testb  $0x1,0x54(%rsp)
   c26a2:	je     c26cc <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x124c>
   c26a4:	cmpl   $0x1,0x50(%rsp)
   c26a9:	jne    c26e0 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1260>
   c26ab:	cmp    $0x5,%cl
   c26ae:	jne    c26f4 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1274>
   c26b0:	mov    $0xa,%ebp
   c26b5:	xor    %r14d,%r14d
   c26b8:	lea    -0xa1bae(%rip),%r12        # 20b11 <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.12047894789963178251+0xa2>
   c26bf:	jmp    c250d <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x108d>
   c26c4:	xor    %r14d,%r14d
   c26c7:	jmp    c250d <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x108d>
   c26cc:	mov    $0x9,%ebp
   c26d1:	xor    %r14d,%r14d
   c26d4:	lea    -0xa1be8(%rip),%r12        # 20af3 <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.12047894789963178251+0x84>
   c26db:	jmp    c250d <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x108d>
   c26e0:	mov    $0x3,%ebp
   c26e5:	xor    %r14d,%r14d
   c26e8:	lea    -0xa1be1(%rip),%r12        # 20b0e <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.12047894789963178251+0x9f>
   c26ef:	jmp    c250d <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x108d>
   c26f4:	mov    0x90(%rsp),%r12
   c26fc:	test   %r12,%r12
   c26ff:	je     c27a5 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1325>
   c2705:	mov    0x80(%rsp),%rax
   c270d:	mov    0x88(%rsp),%r13
   c2715:	mov    0x68(%rsp),%rbp
   c271a:	cmp    %r15,%rax
   c271d:	jne    c27b4 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1334>
   c2723:	mov    0x70(%rsp),%r14
   c2728:	jmp    c254f <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x10cf>
   c272d:	lea    -0xa19f4(%rip),%rax        # 20d40 <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.12047894789963178251+0x2d1>
   c2734:	mov    $0x2,%r13d
   c273a:	jmp    c2500 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1080>
   c273f:	mov    $0x5,%r13d
   c2745:	mov    $0xb,%ebp
   c274a:	lea    -0xa1b0d(%rip),%rax        # 20c44 <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.12047894789963178251+0x1d5>
   c2751:	jmp    c2500 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1080>
   c2756:	lea    -0xa1b0e(%rip),%rax        # 20c4f <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.12047894789963178251+0x1e0>
   c275d:	mov    $0x8,%r13d
   c2763:	jmp    c2500 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1080>
   c2768:	mov    $0x1a,%ebp
   c276d:	lea    -0xa1a4e(%rip),%rax        # 20d26 <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.12047894789963178251+0x2b7>
   c2774:	mov    $0x2,%r13d
   c277a:	jmp    c2500 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1080>
   c277f:	lea    -0xa1aab(%rip),%rax        # 20cdb <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.12047894789963178251+0x26c>
   c2786:	xor    %r13d,%r13d
   c2789:	jmp    c2500 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1080>
   c278e:	mov    $0x4,%r13d
   c2794:	mov    $0x19,%ebp
   c2799:	lea    -0xa1b43(%rip),%rax        # 20c5d <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.12047894789963178251+0x1ee>
   c27a0:	jmp    c2500 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1080>
   c27a5:	lea    -0xa1c91(%rip),%r12        # 20b1b <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.12047894789963178251+0xac>
   c27ac:	xor    %r14d,%r14d
   c27af:	jmp    c250d <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x108d>
   c27b4:	movups 0x70(%rsp),%xmm0
   c27b9:	mov    %rax,(%rbx)
   c27bc:	mov    %r13,0x8(%rbx)
   c27c0:	mov    %r12,0x10(%rbx)
   c27c4:	mov    %rbp,0x18(%rbx)
   c27c8:	movups %xmm0,0x20(%rbx)
   c27cc:	mov    0x60(%rsp),%rax
   c27d1:	mov    %rax,0x30(%rbx)
   c27d5:	mov    0xac(%rsp),%eax
   c27dc:	mov    %eax,0x38(%rbx)
   c27df:	mov    %cl,0x3c(%rbx)
   c27e2:	jmp    c2562 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x10e2>
   c27e7:	lea    0x1e90ba(%rip),%rcx        # 2ab8a8 <anon.49e524b3d56d2aeb6c463ad9e106202e.63.llvm.12047894789963178251+0x30>
   c27ee:	mov    $0x6,%edx
   c27f3:	xor    %edi,%edi
   c27f5:	mov    %r12,%rsi
   c27f8:	call   *0x1f5342(%rip)        # 2b7b40 <_DYNAMIC+0x498>
   c27fe:	ud2
   c2800:	lea    -0xa1c99(%rip),%rax        # 20b6e <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.12047894789963178251+0xff>
   c2807:	jmp    c2500 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1080>
   c280c:	lea    -0xa1d17(%rip),%rdx        # 20afc <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.12047894789963178251+0x8d>
   c2813:	mov    $0x12,%ecx
   c2818:	mov    %rdi,%r15
   c281b:	mov    %r9,%r12
   c281e:	mov    %r9,%rsi
   c2821:	call   c2ad0 <rvvdk_vmdk::descriptor::eq>
   c2826:	test   %al,%al
   c2828:	je     c285b <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x13db>
   c282a:	lea    -0xa1ccf(%rip),%rax        # 20b62 <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.12047894789963178251+0xf3>
   c2831:	mov    $0xc,%ebp
   c2836:	mov    $0x5,%r13d
   c283c:	jmp    c2500 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1080>
   c2841:	mov    $0x3,%r13d
   c2847:	jmp    c24e1 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1061>
   c284c:	mov    0x48(%rsp),%r14
   c2851:	mov    0x38(%rsp),%rbx
   c2856:	jmp    c2500 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1080>
   c285b:	lea    -0xa1d76(%rip),%rdx        # 20aec <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.12047894789963178251+0x7d>
   c2862:	mov    $0x7,%ecx
   c2867:	mov    %r15,%rdi
   c286a:	mov    %r12,%rsi
   c286d:	call   c2ad0 <rvvdk_vmdk::descriptor::eq>
   c2872:	test   %al,%al
   c2874:	jne    c28e6 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1466>
   c2876:	lea    -0xa1d6f(%rip),%rdx        # 20b0e <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.12047894789963178251+0x9f>
   c287d:	mov    $0x3,%ecx
   c2882:	mov    %r15,%rdi
   c2885:	mov    %r12,%rsi
   c2888:	call   c2ad0 <rvvdk_vmdk::descriptor::eq>
   c288d:	test   %al,%al
   c288f:	jne    c28e6 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1466>
   c2891:	lea    -0xa1da5(%rip),%rdx        # 20af3 <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.12047894789963178251+0x84>
   c2898:	mov    $0x9,%ecx
   c289d:	mov    %r15,%rdi
   c28a0:	mov    %r12,%rsi
   c28a3:	call   c2ad0 <rvvdk_vmdk::descriptor::eq>
   c28a8:	test   %al,%al
   c28aa:	jne    c28e6 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1466>
   c28ac:	lea    -0xa1da2(%rip),%rdx        # 20b11 <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.12047894789963178251+0xa2>
   c28b3:	mov    $0xa,%ecx
   c28b8:	mov    %r15,%rdi
   c28bb:	mov    %r12,%rsi
   c28be:	call   c2ad0 <rvvdk_vmdk::descriptor::eq>
   c28c3:	test   %al,%al
   c28c5:	jne    c28e6 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1466>
   c28c7:	lea    -0xab336(%rip),%rdx        # 17598 <anon.786f9ca1aa6bf68944dde0bec4ae4f7b.2.llvm.4745231054059156856+0x2d8>
   c28ce:	mov    $0x8,%ecx
   c28d3:	mov    %r15,%rdi
   c28d6:	mov    %r12,%rsi
   c28d9:	call   c2ad0 <rvvdk_vmdk::descriptor::eq>
   c28de:	test   %al,%al
   c28e0:	je     c29e0 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1560>
   c28e6:	lea    -0xa1d9f(%rip),%rax        # 20b4e <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.12047894789963178251+0xdf>
   c28ed:	jmp    c2500 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1080>
   c28f2:	mov    0x10(%rsp),%rax
   c28f7:	jmp    c24e1 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1061>
   c28fc:	mov    0x128(%rsp),%rax
   c2904:	mov    0x130(%rsp),%rbp
   c290c:	mov    0x138(%rsp),%r14
   c2914:	jmp    c2500 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1080>
   c2919:	mov    $0x1a,%ebp
   c291e:	lea    -0xa1c87(%rip),%rax        # 20c9e <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.12047894789963178251+0x22f>
   c2925:	mov    $0x8,%r13d
   c292b:	jmp    c2500 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1080>
   c2930:	xor    $0x1,%al
   c2932:	movzbl %al,%eax
   c2935:	lea    0x3(,%rax,2),%r13
   c293d:	mov    $0x8,%ebp
   c2942:	lea    -0xab3b1(%rip),%rax        # 17598 <anon.786f9ca1aa6bf68944dde0bec4ae4f7b.2.llvm.4745231054059156856+0x2d8>
   c2949:	jmp    c2500 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1080>
   c294e:	mov    $0x23,%ebp
   c2953:	lea    -0xa1ca2(%rip),%rax        # 20cb8 <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.12047894789963178251+0x249>
   c295a:	mov    $0x8,%r13d
   c2960:	jmp    c2500 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1080>
   c2965:	mov    $0x5,%r13d
   c296b:	mov    $0xc,%ebp
   c2970:	lea    -0xa1e15(%rip),%rax        # 20b62 <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.12047894789963178251+0xf3>
   c2977:	jmp    c2500 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1080>
   c297c:	mov    $0x5,%r13d
   c2982:	mov    $0x12,%ebp
   c2987:	lea    -0xa1e6c(%rip),%rax        # 20b22 <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.12047894789963178251+0xb3>
   c298e:	jmp    c2500 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1080>
   c2993:	mov    $0x12,%ebp
   c2998:	lea    -0xa1e1d(%rip),%rax        # 20b82 <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.12047894789963178251+0x113>
   c299f:	jmp    c284c <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x13cc>
   c29a4:	mov    $0x18,%ebp
   c29a9:	lea    -0xa1da5(%rip),%rax        # 20c0b <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.12047894789963178251+0x19c>
   c29b0:	jmp    c284c <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x13cc>
   c29b5:	mov    $0x5,%r13d
   c29bb:	mov    $0xb,%ebp
   c29c0:	lea    -0xa1e8e(%rip),%rax        # 20b39 <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.12047894789963178251+0xca>
   c29c7:	jmp    c2500 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1080>
   c29cc:	mov    $0x10,%ebp
   c29d1:	lea    -0xab988(%rip),%rax        # 17050 <anon.83d0f48a2d0b5ff7b6e24226a84ebc25.25.llvm.569205349683904225+0x20>
   c29d8:	xor    %r13d,%r13d
   c29db:	jmp    c284c <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x13cc>
   c29e0:	lea    -0xa1ea3(%rip),%rax        # 20b44 <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.12047894789963178251+0xd5>
   c29e7:	mov    $0xa,%ebp
   c29ec:	mov    $0x5,%r13d
   c29f2:	jmp    c2500 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1080>
   c29f7:	mov    %r9,%rax
   c29fa:	jmp    c24e1 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1061>
   c29ff:	mov    $0x5,%r13d
   c2a05:	mov    $0x7,%ebp
   c2a0a:	lea    -0xa1e0d(%rip),%rax        # 20c04 <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.12047894789963178251+0x195>
   c2a11:	jmp    c284c <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x13cc>
   c2a16:	jmp    c2a1a <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x159a>
   c2a18:	jmp    c2a1a <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x159a>
   c2a1a:	mov    %rax,%rbx
   c2a1d:	mov    0x68(%rsp),%rsi
   c2a22:	test   %rsi,%rsi
   c2a25:	jne    c2a3c <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x15bc>
   c2a27:	mov    0x80(%rsp),%rax
   c2a2f:	test   %rax,%rax
   c2a32:	jne    c2a5d <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x15dd>
   c2a34:	mov    %rbx,%rdi
   c2a37:	call   2a9510 <_Unwind_Resume@plt>
   c2a3c:	mov    0x70(%rsp),%rdi
   c2a41:	shl    $0x5,%rsi
   c2a45:	mov    $0x8,%edx
   c2a4a:	call   *0x1f4ea8(%rip)        # 2b78f8 <_DYNAMIC+0x250>
   c2a50:	mov    0x80(%rsp),%rax
   c2a58:	test   %rax,%rax
   c2a5b:	je     c2a34 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x15b4>
   c2a5d:	mov    0x88(%rsp),%rdi
   c2a65:	imul   $0x38,%rax,%rsi
   c2a69:	mov    $0x8,%edx
   c2a6e:	call   *0x1f4e84(%rip)        # 2b78f8 <_DYNAMIC+0x250>
   c2a74:	mov    %rbx,%rdi
   c2a77:	call   2a9510 <_Unwind_Resume@plt>
