
target/r510-reference/descriptor-baseline:     file format elf64-x86-64


Disassembly of section .text:

00000000000c1450 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits>:
   c1450:	push   %rbp
   c1451:	push   %r15
   c1453:	push   %r14
   c1455:	push   %r13
   c1457:	push   %r12
   c1459:	push   %rbx
   c145a:	sub    $0x1f8,%rsp
   c1461:	mov    %rdi,%rbx
   c1464:	movabs $0x8000000000000000,%r15
   c146e:	cmp    (%rcx),%rdx
   c1471:	jbe    c148a <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x3a>
   c1473:	lea    -0xaa3ea(%rip),%r12        # 17090 <anon.83d0f48a2d0b5ff7b6e24226a84ebc25.25.llvm.569205349683904225+0x60>
   c147a:	mov    $0x10,%ebp
   c147f:	xor    %r13d,%r13d
   c1482:	xor    %r14d,%r14d
   c1485:	jmp    c251f <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x10cf>
   c148a:	mov    %rcx,%r14
   c148d:	lea    0x120(%rsp),%rdi
   c1495:	call   *0x1f69bd(%rip)        # 2b7e58 <_DYNAMIC+0x7e0>
   c149b:	cmpl   $0x1,0x120(%rsp)
   c14a3:	jne    c14b3 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x63>
   c14a5:	mov    $0x1,%r13d
   c14ab:	xor    %r14d,%r14d
   c14ae:	jmp    c251f <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x10cf>
   c14b3:	mov    0x128(%rsp),%rax
   c14bb:	mov    0x130(%rsp),%rcx
   c14c3:	movq   $0x0,0x80(%rsp)
   c14cf:	movq   $0x8,0x88(%rsp)
   c14db:	movq   $0x0,0x90(%rsp)
   c14e7:	movq   $0x0,0x68(%rsp)
   c14f0:	movq   $0x8,0x70(%rsp)
   c14f9:	movq   $0x0,0x78(%rsp)
   c1502:	xorps  %xmm0,%xmm0
   c1505:	movaps %xmm0,0xb0(%rsp)
   c150d:	mov    %rcx,0xc0(%rsp)
   c1515:	lea    0xc8(%rsp),%rsi
   c151d:	mov    %rax,0xc8(%rsp)
   c1525:	mov    %rcx,0xd0(%rsp)
   c152d:	movq   $0x0,0xd8(%rsp)
   c1539:	mov    %rcx,0xe0(%rsp)
   c1541:	movabs $0xa0000000a,%rax
   c154b:	mov    %rax,0xe8(%rsp)
   c1553:	movb   $0x1,0xf0(%rsp)
   c155b:	movw   $0x1,0xf8(%rsp)
   c1565:	mov    0x8(%r14),%rax
   c1569:	mov    %rax,0x1f0(%rsp)
   c1571:	mov    0x10(%r14),%rax
   c1575:	mov    %rax,0x1e8(%rsp)
   c157d:	mov    0x20(%r14),%rax
   c1581:	mov    %rax,0x1e0(%rsp)
   c1589:	mov    0x18(%r14),%rax
   c158d:	mov    %rax,0x1d8(%rsp)
   c1595:	mov    $0x5,%bpl
   c1598:	lea    0x120(%rsp),%rdi
   c15a0:	lea    -0xa08b1(%rip),%r13        # 20cf6 <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.7926015133572591147+0x287>
   c15a7:	movl   $0x0,0x54(%rsp)
   c15af:	movl   $0x0,0x50(%rsp)
   c15b7:	movq   $0x0,0xa0(%rsp)
   c15c3:	movq   $0x0,0x118(%rsp)
   c15cf:	movl   $0x0,0x18(%rsp)
   c15d7:	movq   $0x0,0x60(%rsp)
   c15e0:	mov    0xc8(%rsp),%r12
   c15e8:	call   c1240 <<core::str::pattern::CharSearcher as core::str::pattern::Searcher>::next_match>
   c15ed:	cmpl   $0x1,0x120(%rsp)
   c15f5:	jne    c161f <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1cf>
   c15f7:	mov    0x128(%rsp),%rax
   c15ff:	mov    0x130(%rsp),%rcx
   c1607:	mov    0xb8(%rsp),%rdx
   c160f:	sub    %rdx,%rax
   c1612:	add    %rdx,%r12
   c1615:	mov    %rcx,0xb8(%rsp)
   c161d:	jmp    c1663 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x213>
   c161f:	cmpb   $0x0,0xf9(%rsp)
   c1627:	jne    c2645 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x11f5>
   c162d:	movb   $0x1,0xf9(%rsp)
   c1635:	mov    0xb8(%rsp),%r12
   c163d:	mov    0xc0(%rsp),%rax
   c1645:	sub    %r12,%rax
   c1648:	setne  %cl
   c164b:	or     0xf8(%rsp),%cl
   c1652:	cmp    $0x1,%cl
   c1655:	jne    c2645 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x11f5>
   c165b:	add    0xc8(%rsp),%r12
   c1663:	mov    0xb0(%rsp),%r14
   c166b:	inc    %r14
   c166e:	mov    %r14,0xb0(%rsp)
   c1676:	cmp    0x1f0(%rsp),%rax
   c167e:	ja     c2547 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x10f7>
   c1684:	xor    %edx,%edx
   c1686:	test   %rax,%rax
   c1689:	je     c1697 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x247>
   c168b:	cmpb   $0xd,-0x1(%r12,%rax,1)
   c1691:	sete   %dl
   c1694:	neg    %rdx
   c1697:	add    %rax,%rdx
   c169a:	lea    (%r12,%rdx,1),%rax
   c169e:	mov    %r12,%rcx
   c16a1:	jmp    c16d9 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x289>
   c16a3:	data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
   c16b0:	inc    %rcx
   c16b3:	lea    -0x7f(%rsi),%edi
   c16b6:	xor    %r8d,%r8d
   c16b9:	cmp    $0x21,%edi
   c16bc:	setb   %r8b
   c16c0:	xor    %edi,%edi
   c16c2:	cmp    $0x9,%esi
   c16c5:	setne  %dil
   c16c9:	cmp    $0x20,%esi
   c16cc:	cmovb  %edi,%r8d
   c16d0:	test   %r8b,%r8b
   c16d3:	jne    c24ab <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x105b>
   c16d9:	cmp    %rax,%rcx
   c16dc:	je     c175a <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x30a>
   c16de:	movzbl (%rcx),%esi
   c16e1:	test   %sil,%sil
   c16e4:	jns    c16b0 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x260>
   c16e6:	mov    %esi,%edi
   c16e8:	and    $0x1f,%edi
   c16eb:	movzbl 0x1(%rcx),%r9d
   c16f0:	and    $0x3f,%r9d
   c16f4:	cmp    $0xdf,%sil
   c16f8:	jbe    c1737 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x2e7>
   c16fa:	movzbl 0x2(%rcx),%r8d
   c16ff:	shl    $0x6,%r9d
   c1703:	and    $0x3f,%r8d
   c1707:	or     %r9d,%r8d
   c170a:	cmp    $0xf0,%sil
   c170e:	jb     c1748 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x2f8>
   c1710:	movzbl 0x3(%rcx),%esi
   c1714:	and    $0x7,%edi
   c1717:	shl    $0x12,%edi
   c171a:	shl    $0x6,%r8d
   c171e:	and    $0x3f,%esi
   c1721:	or     %r8d,%esi
   c1724:	or     %edi,%esi
   c1726:	cmp    $0x110000,%esi
   c172c:	je     c175a <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x30a>
   c172e:	add    $0x4,%rcx
   c1732:	jmp    c16b3 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x263>
   c1737:	add    $0x2,%rcx
   c173b:	shl    $0x6,%edi
   c173e:	or     %r9d,%edi
   c1741:	mov    %edi,%esi
   c1743:	jmp    c16b3 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x263>
   c1748:	add    $0x3,%rcx
   c174c:	shl    $0xc,%edi
   c174f:	or     %edi,%r8d
   c1752:	mov    %r8d,%esi
   c1755:	jmp    c16b3 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x263>
   c175a:	lea    0x120(%rsp),%rdi
   c1762:	mov    %r12,%rsi
   c1765:	mov    %r14,%rcx
   c1768:	call   c2d30 <rvvdk_vmdk::descriptor::tokens>
   c176d:	mov    %bpl,0x7(%rsp)
   c1772:	mov    0x120(%rsp),%rax
   c177a:	mov    0x128(%rsp),%rdi
   c1782:	mov    0x130(%rsp),%r9
   c178a:	mov    0x138(%rsp),%rbp
   c1792:	mov    0x140(%rsp),%r13
   c179a:	cmp    $0x3,%rax
   c179e:	je     c2561 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1111>
   c17a4:	mov    0x1b0(%rsp),%r12
   c17ac:	cmp    $0x7,%r12
   c17b0:	jae    c27b7 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1367>
   c17b6:	mov    0x148(%rsp),%r15
   c17be:	mov    0x150(%rsp),%rcx
   c17c6:	mov    %rcx,0x58(%rsp)
   c17cb:	mov    0x158(%rsp),%rdx
   c17d3:	mov    0x160(%rsp),%r8
   c17db:	mov    0x168(%rsp),%rcx
   c17e3:	mov    %rcx,0x48(%rsp)
   c17e8:	mov    0x170(%rsp),%r11
   c17f0:	mov    0x178(%rsp),%r10
   c17f8:	mov    0x180(%rsp),%rcx
   c1800:	mov    %rcx,0x1d0(%rsp)
   c1808:	mov    0x188(%rsp),%rcx
   c1810:	mov    %rcx,0x1c0(%rsp)
   c1818:	mov    0x190(%rsp),%rcx
   c1820:	mov    %rcx,0x1c8(%rsp)
   c1828:	cmp    $0x3,%r12
   c182c:	je     c1873 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x423>
   c182e:	test   %r12,%r12
   c1831:	lea    0xc8(%rsp),%rsi
   c1839:	jne    c1883 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x433>
   c183b:	movabs $0x8000000000000000,%r15
   c1845:	lea    -0xa0b56(%rip),%r13        # 20cf6 <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.7926015133572591147+0x287>
   c184c:	lea    -0xa0d67(%rip),%r12        # 20aec <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.7926015133572591147+0x7d>
   c1853:	movzbl 0x7(%rsp),%ebp
   c1858:	cmpb   $0x0,0xf9(%rsp)
   c1860:	lea    0x120(%rsp),%rdi
   c1868:	je     c15e0 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x190>
   c186e:	jmp    c2656 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1206>
   c1873:	mov    %rbp,%rcx
   c1876:	xor    $0x2,%rcx
   c187a:	or     %rax,%rcx
   c187d:	je     c1ce7 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x897>
   c1883:	cmpl   $0x2,0x18(%rsp)
   c1888:	je     c256f <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x111f>
   c188e:	mov    0x90(%rsp),%rcx
   c1896:	mov    %rcx,0x18(%rsp)
   c189b:	cmp    0x1e8(%rsp),%rcx
   c18a3:	jae    c2586 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1136>
   c18a9:	test   %rax,%rax
   c18ac:	jne    c259a <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x114a>
   c18b2:	cmp    $0x3,%r12
   c18b6:	jb     c259a <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x114a>
   c18bc:	test   %rbp,%rbp
   c18bf:	jne    c259a <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x114a>
   c18c5:	cmpq   $0x0,0x58(%rsp)
   c18cb:	jne    c259a <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x114a>
   c18d1:	mov    %r11,0x1b8(%rsp)
   c18d9:	mov    %r10,0x58(%rsp)
   c18de:	mov    %rbx,0x38(%rsp)
   c18e3:	mov    %r14,%rbx
   c18e6:	mov    %r8,0x28(%rsp)
   c18eb:	mov    %rdx,0x30(%rsp)
   c18f0:	mov    $0x2,%ecx
   c18f5:	mov    %r9,%rsi
   c18f8:	lea    -0xa0cdc(%rip),%rdx        # 20c23 <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.7926015133572591147+0x1b4>
   c18ff:	mov    %rdi,%rbp
   c1902:	mov    %r9,%r14
   c1905:	call   c2aa0 <rvvdk_vmdk::descriptor::eq>
   c190a:	mov    %eax,%edx
   c190c:	test   %al,%al
   c190e:	jne    c1933 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x4e3>
   c1910:	mov    $0x6,%ecx
   c1915:	mov    %rbp,%rdi
   c1918:	mov    %r14,%rsi
   c191b:	mov    %edx,%ebp
   c191d:	lea    -0xa0cff(%rip),%rdx        # 20c25 <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.7926015133572591147+0x1b6>
   c1924:	call   c2aa0 <rvvdk_vmdk::descriptor::eq>
   c1929:	mov    %ebp,%edx
   c192b:	test   %al,%al
   c192d:	je     c25be <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x116e>
   c1933:	test   %r15,%r15
   c1936:	je     c25b1 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1161>
   c193c:	add    $0xfffffffffffffffd,%r12
   c1940:	xor    $0x1,%dl
   c1943:	xor    %eax,%eax
   c1945:	mov    %rbx,%r14
   c1948:	nopl   0x0(%rax,%rax,1)
   c1950:	cmp    %rax,%r15
   c1953:	je     c196b <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x51b>
   c1955:	movzbl 0x0(%r13,%rax,1),%ecx
   c195b:	add    $0xc6,%cl
   c195e:	inc    %rax
   c1961:	cmp    $0xf6,%cl
   c1964:	jae    c1950 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x500>
   c1966:	jmp    c24b3 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1063>
   c196b:	movzbl 0x0(%r13),%eax
   c1970:	cmp    $0x1,%r15
   c1974:	mov    0x38(%rsp),%rbx
   c1979:	jne    c198d <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x53d>
   c197b:	cmp    $0x2b,%eax
   c197e:	je     c24c5 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1075>
   c1984:	cmp    $0x2d,%eax
   c1987:	je     c24c5 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1075>
   c198d:	mov    %dl,0x8(%rsp)
   c1991:	xor    %edx,%edx
   c1993:	cmp    $0x2b,%eax
   c1996:	sete   %dl
   c1999:	mov    %r15,%rcx
   c199c:	sub    %rdx,%rcx
   c199f:	add    %rdx,%r13
   c19a2:	mov    %rdx,%rax
   c19a5:	neg    %rax
   c19a8:	cmp    $0x11,%rcx
   c19ac:	jae    c1ae0 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x690>
   c19b2:	test   %rcx,%rcx
   c19b5:	je     c25dd <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x118d>
   c19bb:	add    %rax,%r15
   c19be:	neg    %r15
   c19c1:	xor    %ecx,%ecx
   c19c3:	xor    %eax,%eax
   c19c5:	data16 cs nopw 0x0(%rax,%rax,1)
   c19d0:	movzbl 0x0(%r13,%rcx,1),%edx
   c19d6:	add    $0xffffffd0,%edx
   c19d9:	cmp    $0x9,%edx
   c19dc:	ja     c24c5 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1075>
   c19e2:	lea    (%rax,%rax,4),%rax
   c19e6:	mov    %edx,%edx
   c19e8:	lea    (%rdx,%rax,2),%rax
   c19ec:	inc    %rcx
   c19ef:	mov    %r15,%rdx
   c19f2:	add    %rcx,%rdx
   c19f5:	jne    c19d0 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x580>
   c19f7:	mov    %rax,%rcx
   c19fa:	shr    $0x37,%rcx
   c19fe:	mov    $0x7,%r13d
   c1a04:	jne    c28c2 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1472>
   c1a0a:	test   %rax,%rax
   c1a0d:	mov    0x30(%rsp),%rbp
   c1a12:	je     c261f <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x11cf>
   c1a18:	mov    %rax,0x40(%rsp)
   c1a1d:	shl    $0x9,%rax
   c1a21:	mov    %rax,0x10(%rsp)
   c1a26:	mov    $0x4,%ecx
   c1a2b:	mov    %rbp,%rdi
   c1a2e:	mov    0x28(%rsp),%r15
   c1a33:	mov    %r15,%rsi
   c1a36:	lea    -0xa2ccd(%rip),%rdx        # 1ed70 <anon.d625489d584c397ac22d75864c33158f.21.llvm.5936746164385555759+0x30>
   c1a3d:	call   c2aa0 <rvvdk_vmdk::descriptor::eq>
   c1a42:	test   %al,%al
   c1a44:	je     c1b28 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x6d8>
   c1a4a:	mov    $0x21,%ebp
   c1a4f:	cmp    $0x2,%r12
   c1a53:	jne    c26fd <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x12ad>
   c1a59:	cmpq   $0x1,0x48(%rsp)
   c1a5f:	mov    0x40(%rsp),%rcx
   c1a64:	mov    0x58(%rsp),%r10
   c1a69:	mov    0x1b8(%rsp),%r11
   c1a71:	jne    c26fd <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x12ad>
   c1a77:	cmpq   $0x0,0x1d0(%rsp)
   c1a80:	jne    c26fd <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x12ad>
   c1a86:	mov    $0xe,%ebp
   c1a8b:	test   %r10,%r10
   c1a8e:	je     c2726 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x12d6>
   c1a94:	cmp    0x1e0(%rsp),%r10
   c1a9c:	ja     c274f <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x12ff>
   c1aa2:	mov    0x1c8(%rsp),%r12
   c1aaa:	test   %r12,%r12
   c1aad:	je     c2557 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1107>
   c1ab3:	xor    %eax,%eax
   c1ab5:	movzbl 0x7(%rsp),%edi
   c1aba:	mov    0x1c0(%rsp),%r15
   c1ac2:	cmp    %rax,%r12
   c1ac5:	je     c1c4e <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x7fe>
   c1acb:	movzbl (%r15,%rax,1),%edx
   c1ad0:	add    $0xc6,%dl
   c1ad3:	inc    %rax
   c1ad6:	cmp    $0xf6,%dl
   c1ad9:	jae    c1ac2 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x672>
   c1adb:	jmp    c2557 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1107>
   c1ae0:	add    %rax,%r15
   c1ae3:	neg    %r15
   c1ae6:	xor    %ecx,%ecx
   c1ae8:	xor    %eax,%eax
   c1aea:	mov    %r15,%rdx
   c1aed:	add    %rcx,%rdx
   c1af0:	je     c19f7 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x5a7>
   c1af6:	mov    $0xa,%edx
   c1afb:	mul    %rdx
   c1afe:	jo     c24c5 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1075>
   c1b04:	movzbl 0x0(%r13,%rcx,1),%esi
   c1b0a:	add    $0xffffffd0,%esi
   c1b0d:	add    %rsi,%rax
   c1b10:	setb   %dl
   c1b13:	cmp    $0x9,%esi
   c1b16:	ja     c24c5 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1075>
   c1b1c:	inc    %rcx
   c1b1f:	test   %dl,%dl
   c1b21:	je     c1aea <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x69a>
   c1b23:	jmp    c24c5 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1075>
   c1b28:	mov    $0x4,%ecx
   c1b2d:	mov    %rbp,%rdi
   c1b30:	mov    %r15,%rsi
   c1b33:	lea    -0xa2e3e(%rip),%rdx        # 1ecfc <anon.83d0f48a2d0b5ff7b6e24226a84ebc25.18.llvm.569205349683904225+0x14>
   c1b3a:	call   c2aa0 <rvvdk_vmdk::descriptor::eq>
   c1b3f:	test   %al,%al
   c1b41:	movzbl 0x7(%rsp),%eax
   c1b46:	je     c270f <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x12bf>
   c1b4c:	test   %r12,%r12
   c1b4f:	jne    c2738 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x12e8>
   c1b55:	mov    $0x1,%r15d
   c1b5b:	cmp    $0x2,%al
   c1b5d:	mov    0x110(%rsp),%rcx
   c1b65:	mov    0x108(%rsp),%rdx
   c1b6d:	jne    c25f4 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x11a4>
   c1b73:	mov    0x10(%rsp),%r12
   c1b78:	add    0x60(%rsp),%r12
   c1b7d:	lea    -0xa0e8e(%rip),%rax        # 20cf6 <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.7926015133572591147+0x287>
   c1b84:	jb     c24d0 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1080>
   c1b8a:	mov    %rdx,0x108(%rsp)
   c1b92:	mov    %rcx,0x110(%rsp)
   c1b9a:	mov    0x18(%rsp),%r14
   c1b9f:	cmp    0x80(%rsp),%r14
   c1ba7:	mov    %rax,%r13
   c1baa:	movzbl 0x7(%rsp),%ebp
   c1baf:	jne    c1bbf <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x76f>
   c1bb1:	lea    0x80(%rsp),%rdi
   c1bb9:	call   *0x1f63b9(%rip)        # 2b7f78 <_DYNAMIC+0x900>
   c1bbf:	mov    0x88(%rsp),%rax
   c1bc7:	imul   $0x38,%r14,%rcx
   c1bcb:	mov    %r15,(%rax,%rcx,1)
   c1bcf:	mov    0x110(%rsp),%rdx
   c1bd7:	mov    %rdx,0x8(%rax,%rcx,1)
   c1bdc:	mov    0x108(%rsp),%rdx
   c1be4:	mov    %rdx,0x10(%rax,%rcx,1)
   c1be9:	mov    0x98(%rsp),%rdx
   c1bf1:	mov    %rdx,0x18(%rax,%rcx,1)
   c1bf6:	mov    0x60(%rsp),%rdx
   c1bfb:	mov    %rdx,0x20(%rax,%rcx,1)
   c1c00:	mov    0x10(%rsp),%rdx
   c1c05:	mov    %rdx,0x28(%rax,%rcx,1)
   c1c0a:	movzbl 0x8(%rsp),%edx
   c1c0f:	mov    %dl,0x30(%rax,%rcx,1)
   c1c13:	inc    %r14
   c1c16:	mov    %r14,0x90(%rsp)
   c1c1e:	movl   $0x1,0x18(%rsp)
   c1c26:	cmpb   $0x0,0xf9(%rsp)
   c1c2e:	mov    %r12,0x60(%rsp)
   c1c33:	lea    0xc8(%rsp),%rsi
   c1c3b:	lea    0x120(%rsp),%rdi
   c1c43:	je     c15e0 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x190>
   c1c49:	jmp    c2640 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x11f0>
   c1c4e:	movzbl (%r15),%eax
   c1c52:	cmp    $0x1,%r12
   c1c56:	jne    c1c6a <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x81a>
   c1c58:	cmp    $0x2b,%eax
   c1c5b:	je     c2557 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1107>
   c1c61:	cmp    $0x2d,%eax
   c1c64:	je     c2557 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1107>
   c1c6a:	xor    %esi,%esi
   c1c6c:	cmp    $0x2b,%eax
   c1c6f:	sete   %sil
   c1c73:	mov    %r12,%rdx
   c1c76:	sub    %rsi,%rdx
   c1c79:	add    %rsi,%r15
   c1c7c:	mov    %rsi,%rax
   c1c7f:	neg    %rax
   c1c82:	cmp    $0x11,%rdx
   c1c86:	jae    c1d18 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x8c8>
   c1c8c:	test   %rdx,%rdx
   c1c8f:	je     c20a1 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0xc51>
   c1c95:	add    %rax,%r12
   c1c98:	neg    %r12
   c1c9b:	xor    %edx,%edx
   c1c9d:	xor    %eax,%eax
   c1c9f:	mov    0x10(%rsp),%r8
   c1ca4:	mov    0x20(%rsp),%r9
   c1ca9:	movzbl (%r15,%rdx,1),%esi
   c1cae:	add    $0xffffffd0,%esi
   c1cb1:	cmp    $0x9,%esi
   c1cb4:	ja     c2617 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x11c7>
   c1cba:	lea    (%rax,%rax,4),%rax
   c1cbe:	mov    %esi,%esi
   c1cc0:	lea    (%rsi,%rax,2),%rax
   c1cc4:	inc    %rdx
   c1cc7:	mov    %r12,%rsi
   c1cca:	add    %rdx,%rsi
   c1ccd:	jne    c1ca9 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x859>
   c1ccf:	movabs $0x7fffffffffffff,%rdx
   c1cd9:	cmp    %rdx,%rax
   c1cdc:	jbe    c20a8 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0xc58>
   c1ce2:	jmp    c29c7 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1577>
   c1ce7:	cmp    $0x4,%r9
   c1ceb:	mov    %rdx,0x30(%rsp)
   c1cf0:	mov    %r8,0x28(%rsp)
   c1cf5:	mov    %r9,0x8(%rsp)
   c1cfa:	jbe    c1d64 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x914>
   c1cfc:	cmpb   $0xc0,0x4(%rdi)
   c1d00:	movabs $0x8000000000000000,%r15
   c1d0a:	lea    -0xa1225(%rip),%r12        # 20aec <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.7926015133572591147+0x7d>
   c1d11:	jge    c1d7b <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x92b>
   c1d13:	jmp    c1f50 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0xb00>
   c1d18:	add    %rax,%r12
   c1d1b:	neg    %r12
   c1d1e:	xor    %esi,%esi
   c1d20:	xor    %eax,%eax
   c1d22:	mov    %r12,%rdx
   c1d25:	add    %rsi,%rdx
   c1d28:	je     c23fc <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0xfac>
   c1d2e:	mov    $0xa,%ecx
   c1d33:	mul    %rcx
   c1d36:	jo     c2557 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1107>
   c1d3c:	movzbl (%r15,%rsi,1),%ecx
   c1d41:	add    $0xffffffd0,%ecx
   c1d44:	add    %rcx,%rax
   c1d47:	setb   %dl
   c1d4a:	cmp    $0x9,%ecx
   c1d4d:	mov    0x40(%rsp),%rcx
   c1d52:	ja     c2557 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1107>
   c1d58:	inc    %rsi
   c1d5b:	test   %dl,%dl
   c1d5d:	je     c1d22 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x8d2>
   c1d5f:	jmp    c2557 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1107>
   c1d64:	movabs $0x8000000000000000,%r15
   c1d6e:	lea    -0xa1289(%rip),%r12        # 20aec <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.7926015133572591147+0x7d>
   c1d75:	jne    c1f50 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0xb00>
   c1d7b:	movzbl (%rdi),%eax
   c1d7e:	lea    -0x41(%rax),%ecx
   c1d81:	cmp    $0x1a,%cl
   c1d84:	setb   %cl
   c1d87:	shl    $0x5,%cl
   c1d8a:	or     %al,%cl
   c1d8c:	cmp    $0x64,%cl
   c1d8f:	jne    c1f50 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0xb00>
   c1d95:	movzbl 0x1(%rdi),%eax
   c1d99:	lea    -0x41(%rax),%ecx
   c1d9c:	cmp    $0x1a,%cl
   c1d9f:	setb   %cl
   c1da2:	shl    $0x5,%cl
   c1da5:	or     %al,%cl
   c1da7:	cmp    $0x64,%cl
   c1daa:	jne    c1f50 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0xb00>
   c1db0:	movzbl 0x2(%rdi),%eax
   c1db4:	lea    -0x41(%rax),%ecx
   c1db7:	cmp    $0x1a,%cl
   c1dba:	setb   %cl
   c1dbd:	shl    $0x5,%cl
   c1dc0:	or     %al,%cl
   c1dc2:	cmp    $0x62,%cl
   c1dc5:	jne    c1f50 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0xb00>
   c1dcb:	movzbl 0x3(%rdi),%eax
   c1dcf:	lea    -0x41(%rax),%ecx
   c1dd2:	cmp    $0x1a,%cl
   c1dd5:	setb   %cl
   c1dd8:	shl    $0x5,%cl
   c1ddb:	or     %al,%cl
   c1ddd:	cmp    $0x2e,%cl
   c1de0:	jne    c1f50 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0xb00>
   c1de6:	mov    %rbx,0x38(%rsp)
   c1deb:	mov    %r14,0x48(%rsp)
   c1df0:	mov    $0x2,%r13d
   c1df6:	cmpl   $0x0,0x18(%rsp)
   c1dfb:	je     c2963 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1513>
   c1e01:	mov    %rdi,%r15
   c1e04:	mov    $0xf,%ecx
   c1e09:	mov    0x8(%rsp),%rsi
   c1e0e:	lea    -0xa1281(%rip),%rdx        # 20b94 <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.7926015133572591147+0x125>
   c1e15:	call   c2aa0 <rvvdk_vmdk::descriptor::eq>
   c1e1a:	test   %al,%al
   c1e1c:	jne    c1ef9 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0xaa9>
   c1e22:	mov    $0x16,%ecx
   c1e27:	mov    %r15,%rdi
   c1e2a:	mov    0x8(%rsp),%rsi
   c1e2f:	lea    -0xa1293(%rip),%rdx        # 20ba3 <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.7926015133572591147+0x134>
   c1e36:	call   c2aa0 <rvvdk_vmdk::descriptor::eq>
   c1e3b:	test   %al,%al
   c1e3d:	jne    c1ef9 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0xaa9>
   c1e43:	mov    $0x12,%ecx
   c1e48:	mov    %r15,%rdi
   c1e4b:	mov    0x8(%rsp),%rsi
   c1e50:	lea    -0xa129e(%rip),%rdx        # 20bb9 <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.7926015133572591147+0x14a>
   c1e57:	call   c2aa0 <rvvdk_vmdk::descriptor::eq>
   c1e5c:	test   %al,%al
   c1e5e:	jne    c1ef9 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0xaa9>
   c1e64:	mov    $0x14,%ecx
   c1e69:	mov    %r15,%rdi
   c1e6c:	mov    0x8(%rsp),%rsi
   c1e71:	lea    -0xa12ad(%rip),%rdx        # 20bcb <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.7926015133572591147+0x15c>
   c1e78:	call   c2aa0 <rvvdk_vmdk::descriptor::eq>
   c1e7d:	test   %al,%al
   c1e7f:	jne    c1ef9 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0xaa9>
   c1e81:	mov    $0x14,%ecx
   c1e86:	mov    %r15,%rdi
   c1e89:	mov    0x8(%rsp),%rsi
   c1e8e:	lea    -0xa12b6(%rip),%rdx        # 20bdf <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.7926015133572591147+0x170>
   c1e95:	call   c2aa0 <rvvdk_vmdk::descriptor::eq>
   c1e9a:	test   %al,%al
   c1e9c:	jne    c1ef9 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0xaa9>
   c1e9e:	mov    $0x10,%ecx
   c1ea3:	mov    %r15,%rdi
   c1ea6:	mov    0x8(%rsp),%rsi
   c1eab:	lea    -0xaab82(%rip),%rdx        # 17330 <anon.786f9ca1aa6bf68944dde0bec4ae4f7b.2.llvm.4745231054059156856+0x70>
   c1eb2:	call   c2aa0 <rvvdk_vmdk::descriptor::eq>
   c1eb7:	test   %al,%al
   c1eb9:	jne    c1ef9 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0xaa9>
   c1ebb:	mov    $0x8,%ecx
   c1ec0:	mov    %r15,%rdi
   c1ec3:	mov    0x8(%rsp),%rsi
   c1ec8:	lea    -0xaa7a7(%rip),%rdx        # 17728 <anon.d625489d584c397ac22d75864c33158f.39.llvm.5936746164385555759+0x20>
   c1ecf:	call   c2aa0 <rvvdk_vmdk::descriptor::eq>
   c1ed4:	test   %al,%al
   c1ed6:	jne    c1ef9 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0xaa9>
   c1ed8:	mov    $0x11,%ecx
   c1edd:	mov    %r15,%rdi
   c1ee0:	mov    0x8(%rsp),%rsi
   c1ee5:	lea    -0xa12f9(%rip),%rdx        # 20bf3 <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.7926015133572591147+0x184>
   c1eec:	call   c2aa0 <rvvdk_vmdk::descriptor::eq>
   c1ef1:	test   %al,%al
   c1ef3:	je     c29cf <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x157f>
   c1ef9:	cmpq   $0x1,0x58(%rsp)
   c1eff:	jne    c2974 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1524>
   c1f05:	mov    0x70(%rsp),%rsi
   c1f0a:	mov    0x78(%rsp),%rbp
   c1f0f:	mov    %rbp,%rbx
   c1f12:	shl    $0x5,%rbp
   c1f16:	mov    %rsi,%r14
   c1f19:	test   %rbp,%rbp
   c1f1c:	mov    0x8(%rsp),%rcx
   c1f21:	je     c241e <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0xfce>
   c1f27:	mov    %r15,%rdx
   c1f2a:	lea    0x20(%rsi),%r12
   c1f2e:	mov    (%rsi),%rdi
   c1f31:	mov    0x8(%rsi),%rsi
   c1f35:	call   c2aa0 <rvvdk_vmdk::descriptor::eq>
   c1f3a:	add    $0xffffffffffffffe0,%rbp
   c1f3e:	mov    $0x3,%r13d
   c1f44:	mov    %r12,%rsi
   c1f47:	test   %al,%al
   c1f49:	je     c1f19 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0xac9>
   c1f4b:	jmp    c281c <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x13cc>
   c1f50:	mov    $0x2,%r13d
   c1f56:	mov    $0x14,%ebp
   c1f5b:	cmpl   $0x0,0x18(%rsp)
   c1f60:	jne    c27d0 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1380>
   c1f66:	mov    0x58(%rsp),%rax
   c1f6b:	test   %rax,%rax
   c1f6e:	mov    %rdi,0x40(%rsp)
   c1f73:	je     c2051 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0xc01>
   c1f79:	cmp    $0x1,%rax
   c1f7d:	jne    c27dc <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x138c>
   c1f83:	mov    $0xa,%ecx
   c1f88:	mov    %r9,%rsi
   c1f8b:	lea    -0xa1481(%rip),%rdx        # 20b11 <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.7926015133572591147+0xa2>
   c1f92:	call   c2aa0 <rvvdk_vmdk::descriptor::eq>
   c1f97:	test   %al,%al
   c1f99:	je     c217a <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0xd2a>
   c1f9f:	mov    $0xe,%ecx
   c1fa4:	mov    0x30(%rsp),%rdi
   c1fa9:	mov    0x28(%rsp),%rsi
   c1fae:	lea    -0xa12b5(%rip),%rdx        # 20d00 <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.7926015133572591147+0x291>
   c1fb5:	call   c2aa0 <rvvdk_vmdk::descriptor::eq>
   c1fba:	movl   $0x0,0x18(%rsp)
   c1fc2:	mov    $0x0,%ebp
   c1fc7:	test   %al,%al
   c1fc9:	lea    -0xa12da(%rip),%r13        # 20cf6 <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.7926015133572591147+0x287>
   c1fd0:	jne    c2039 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0xbe9>
   c1fd2:	mov    $0x12,%ecx
   c1fd7:	mov    0x30(%rsp),%rdi
   c1fdc:	mov    0x28(%rsp),%rsi
   c1fe1:	lea    -0xa12da(%rip),%rdx        # 20d0e <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.7926015133572591147+0x29f>
   c1fe8:	call   c2aa0 <rvvdk_vmdk::descriptor::eq>
   c1fed:	mov    $0x1,%bpl
   c1ff0:	test   %al,%al
   c1ff2:	jne    c2039 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0xbe9>
   c1ff4:	mov    $0x10,%ecx
   c1ff9:	mov    0x30(%rsp),%rdi
   c1ffe:	mov    0x28(%rsp),%rsi
   c2003:	lea    -0xab30a(%rip),%rdx        # 16d00 <anon.e9ff7b7b9ea76053b3f0064765461615.59.llvm.11370392988746934037+0xb0>
   c200a:	call   c2aa0 <rvvdk_vmdk::descriptor::eq>
   c200f:	test   %al,%al
   c2011:	jne    c2039 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0xbe9>
   c2013:	mov    $0x6,%ecx
   c2018:	mov    0x30(%rsp),%rdi
   c201d:	mov    0x28(%rsp),%rsi
   c2022:	lea    -0xa1309(%rip),%rdx        # 20d20 <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.7926015133572591147+0x2b1>
   c2029:	call   c2aa0 <rvvdk_vmdk::descriptor::eq>
   c202e:	mov    $0x2,%bpl
   c2031:	test   %al,%al
   c2033:	je     c2985 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1535>
   c2039:	cmpb   $0x5,0x7(%rsp)
   c203e:	lea    0xc8(%rsp),%rsi
   c2046:	je     c1858 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x408>
   c204c:	jmp    c2811 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x13c1>
   c2051:	mov    $0x7,%ecx
   c2056:	mov    %r9,%rsi
   c2059:	mov    %r12,%rdx
   c205c:	call   c2aa0 <rvvdk_vmdk::descriptor::eq>
   c2061:	test   %al,%al
   c2063:	je     c20e9 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0xc99>
   c2069:	mov    $0x6,%r13d
   c206f:	mov    0x28(%rsp),%rsi
   c2074:	test   %rsi,%rsi
   c2077:	je     c2636 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x11e6>
   c207d:	xor    %eax,%eax
   c207f:	mov    0x30(%rsp),%rdx
   c2084:	cmp    %rax,%rsi
   c2087:	je     c22f9 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0xea9>
   c208d:	movzbl (%rdx,%rax,1),%ecx
   c2091:	add    $0xc6,%cl
   c2094:	inc    %rax
   c2097:	cmp    $0xf6,%cl
   c209a:	jae    c2084 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0xc34>
   c209c:	jmp    c2636 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x11e6>
   c20a1:	xor    %eax,%eax
   c20a3:	mov    0x10(%rsp),%r8
   c20a8:	mov    %rax,%rsi
   c20ab:	shl    $0x9,%rsi
   c20af:	mov    %rsi,%rdx
   c20b2:	add    %r8,%rdx
   c20b5:	jb     c24b1 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1061>
   c20bb:	cmp    $0x2,%dil
   c20bf:	jae    c2201 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0xdb1>
   c20c5:	test   %rax,%rax
   c20c8:	jne    c2600 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x11b0>
   c20ce:	test   %dil,%dil
   c20d1:	je     c221d <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0xdcd>
   c20d7:	cmp    $0x400000,%rcx
   c20de:	jbe    c2229 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0xdd9>
   c20e4:	jmp    c28e9 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1499>
   c20e9:	mov    $0x3,%ecx
   c20ee:	mov    0x40(%rsp),%rdi
   c20f3:	mov    0x8(%rsp),%rsi
   c20f8:	lea    -0xa15f1(%rip),%rdx        # 20b0e <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.7926015133572591147+0x9f>
   c20ff:	call   c2aa0 <rvvdk_vmdk::descriptor::eq>
   c2104:	test   %al,%al
   c2106:	je     c225e <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0xe0e>
   c210c:	lea    0x120(%rsp),%rdi
   c2114:	mov    0x30(%rsp),%rsi
   c2119:	mov    0x28(%rsp),%rdx
   c211e:	mov    %r14,%rcx
   c2121:	call   c2bf0 <rvvdk_vmdk::descriptor::cid>
   c2126:	mov    0x120(%rsp),%r13
   c212e:	cmp    $0x9,%r13
   c2132:	movzbl 0x7(%rsp),%ebp
   c2137:	jne    c28cc <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x147c>
   c213d:	cmpl   $0x1,0x50(%rsp)
   c2142:	lea    0xc8(%rsp),%rsi
   c214a:	je     c2811 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x13c1>
   c2150:	mov    0x128(%rsp),%eax
   c2157:	mov    %eax,0xac(%rsp)
   c215e:	movl   $0x1,0x50(%rsp)
   c2166:	movl   $0x0,0x18(%rsp)
   c216e:	lea    -0xa147f(%rip),%r13        # 20cf6 <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.7926015133572591147+0x287>
   c2175:	jmp    c1858 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x408>
   c217a:	mov    $0x8,%ecx
   c217f:	mov    0x40(%rsp),%rdi
   c2184:	mov    0x8(%rsp),%rsi
   c2189:	lea    -0xaabf8(%rip),%rdx        # 17598 <anon.786f9ca1aa6bf68944dde0bec4ae4f7b.2.llvm.4745231054059156856+0x2d8>
   c2190:	call   c2aa0 <rvvdk_vmdk::descriptor::eq>
   c2195:	test   %al,%al
   c2197:	mov    0x40(%rsp),%rdi
   c219c:	mov    0x8(%rsp),%r9
   c21a1:	je     c27dc <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x138c>
   c21a7:	mov    $0x5,%ecx
   c21ac:	mov    0x30(%rsp),%rdi
   c21b1:	mov    0x28(%rsp),%rsi
   c21b6:	lea    -0xa167d(%rip),%rdx        # 20b40 <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.7926015133572591147+0xd1>
   c21bd:	call   c2aa0 <rvvdk_vmdk::descriptor::eq>
   c21c2:	mov    %eax,%ecx
   c21c4:	not    %cl
   c21c6:	or     0x118(%rsp),%cl
   c21cd:	test   $0x1,%cl
   c21d0:	jne    c2900 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x14b0>
   c21d6:	mov    $0x1,%al
   c21d8:	mov    %rax,0x118(%rsp)
   c21e0:	movl   $0x0,0x18(%rsp)
   c21e8:	lea    -0xa14f9(%rip),%r13        # 20cf6 <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.7926015133572591147+0x287>
   c21ef:	movzbl 0x7(%rsp),%ebp
   c21f4:	lea    0xc8(%rsp),%rsi
   c21fc:	jmp    c1858 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x408>
   c2201:	mov    %rsi,0x20(%rsp)
   c2206:	movzbl %dil,%eax
   c220a:	cmp    $0x2,%eax
   c220d:	jne    c275e <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x130e>
   c2213:	xor    %r15d,%r15d
   c2216:	mov    0x20(%rsp),%rcx
   c221b:	jmp    c2237 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0xde7>
   c221d:	cmpq   $0x0,0x18(%rsp)
   c2223:	jne    c291e <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x14ce>
   c2229:	movq   $0x0,0x20(%rsp)
   c2232:	xor    %r15d,%r15d
   c2235:	xor    %ecx,%ecx
   c2237:	mov    %r11,%rdx
   c223a:	mov    %r10,0x98(%rsp)
   c2242:	mov    0x10(%rsp),%r12
   c2247:	add    0x60(%rsp),%r12
   c224c:	lea    -0xa155d(%rip),%rax        # 20cf6 <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.7926015133572591147+0x287>
   c2253:	jae    c1b8a <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x73a>
   c2259:	jmp    c24d0 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1080>
   c225e:	mov    $0x9,%ecx
   c2263:	mov    0x40(%rsp),%rdi
   c2268:	mov    0x8(%rsp),%rsi
   c226d:	lea    -0xa1781(%rip),%rdx        # 20af3 <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.7926015133572591147+0x84>
   c2274:	call   c2aa0 <rvvdk_vmdk::descriptor::eq>
   c2279:	test   %al,%al
   c227b:	mov    0x40(%rsp),%rdi
   c2280:	mov    0x8(%rsp),%r9
   c2285:	je     c27dc <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x138c>
   c228b:	lea    0x120(%rsp),%rdi
   c2293:	mov    0x30(%rsp),%rsi
   c2298:	mov    0x28(%rsp),%rdx
   c229d:	mov    %r14,%rcx
   c22a0:	call   c2bf0 <rvvdk_vmdk::descriptor::cid>
   c22a5:	mov    0x120(%rsp),%r13
   c22ad:	cmp    $0x9,%r13
   c22b1:	jne    c28cc <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x147c>
   c22b7:	cmpl   $0xffffffff,0x128(%rsp)
   c22bf:	lea    -0xa15d0(%rip),%r13        # 20cf6 <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.7926015133572591147+0x287>
   c22c6:	movzbl 0x7(%rsp),%ebp
   c22cb:	lea    0xc8(%rsp),%rsi
   c22d3:	jne    c2935 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x14e5>
   c22d9:	cmpl   $0x1,0x54(%rsp)
   c22de:	je     c2811 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x13c1>
   c22e4:	movl   $0x1,0x54(%rsp)
   c22ec:	movl   $0x0,0x18(%rsp)
   c22f4:	jmp    c1858 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x408>
   c22f9:	movzbl (%rdx),%eax
   c22fc:	cmp    $0x1,%rsi
   c2300:	jne    c2314 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0xec4>
   c2302:	cmp    $0x2b,%eax
   c2305:	je     c2636 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x11e6>
   c230b:	cmp    $0x2d,%eax
   c230e:	je     c2636 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x11e6>
   c2314:	xor    %ecx,%ecx
   c2316:	cmp    $0x2b,%eax
   c2319:	sete   %cl
   c231c:	mov    %rsi,%rax
   c231f:	sub    %rcx,%rax
   c2322:	add    %rcx,%rdx
   c2325:	neg    %rcx
   c2328:	cmp    $0x11,%rax
   c232c:	jae    c23b6 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0xf66>
   c2332:	test   %rax,%rax
   c2335:	je     c294c <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x14fc>
   c233b:	mov    %rdx,%rdi
   c233e:	add    0x28(%rsp),%rcx
   c2343:	neg    %rcx
   c2346:	xor    %edx,%edx
   c2348:	xor    %eax,%eax
   c234a:	movzbl (%rdi,%rdx,1),%esi
   c234e:	add    $0xffffffd0,%esi
   c2351:	cmp    $0x9,%esi
   c2354:	ja     c2636 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x11e6>
   c235a:	lea    (%rax,%rax,4),%rax
   c235e:	mov    %esi,%esi
   c2360:	lea    (%rsi,%rax,2),%rax
   c2364:	inc    %rdx
   c2367:	mov    %rcx,%rsi
   c236a:	add    %rdx,%rsi
   c236d:	jne    c234a <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0xefa>
   c236f:	cmp    $0x1,%rax
   c2373:	jne    c294c <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x14fc>
   c2379:	testb  $0x1,0xa0(%rsp)
   c2381:	jne    c2811 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x13c1>
   c2387:	mov    $0x1,%al
   c2389:	mov    %rax,0xa0(%rsp)
   c2391:	movl   $0x0,0x18(%rsp)
   c2399:	movabs $0x8000000000000000,%r15
   c23a3:	lea    -0xa16b4(%rip),%r13        # 20cf6 <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.7926015133572591147+0x287>
   c23aa:	lea    -0xa18c5(%rip),%r12        # 20aec <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.7926015133572591147+0x7d>
   c23b1:	jmp    c21ef <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0xd9f>
   c23b6:	mov    %rdx,%r8
   c23b9:	add    %rsi,%rcx
   c23bc:	neg    %rcx
   c23bf:	xor    %esi,%esi
   c23c1:	xor    %eax,%eax
   c23c3:	mov    %rcx,%rdx
   c23c6:	add    %rsi,%rdx
   c23c9:	je     c236f <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0xf1f>
   c23cb:	mov    $0xa,%edx
   c23d0:	mul    %rdx
   c23d3:	jo     c2636 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x11e6>
   c23d9:	movzbl (%r8,%rsi,1),%edi
   c23de:	add    $0xffffffd0,%edi
   c23e1:	add    %rdi,%rax
   c23e4:	setb   %dl
   c23e7:	cmp    $0x9,%edi
   c23ea:	ja     c2636 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x11e6>
   c23f0:	inc    %rsi
   c23f3:	test   %dl,%dl
   c23f5:	je     c23c3 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0xf73>
   c23f7:	jmp    c2636 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x11e6>
   c23fc:	mov    0x10(%rsp),%r8
   c2401:	mov    0x20(%rsp),%r9
   c2406:	movabs $0x7fffffffffffff,%rdx
   c2410:	cmp    %rdx,%rax
   c2413:	jbe    c20a8 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0xc58>
   c2419:	jmp    c29c7 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1577>
   c241e:	cmp    0x1d8(%rsp),%rbx
   c2426:	jae    c299c <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x154c>
   c242c:	cmp    0x68(%rsp),%rbx
   c2431:	jne    c2443 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0xff3>
   c2433:	lea    0x68(%rsp),%rdi
   c2438:	call   *0x1f5b42(%rip)        # 2b7f80 <_DYNAMIC+0x908>
   c243e:	mov    0x70(%rsp),%r14
   c2443:	mov    %rbx,%rax
   c2446:	shl    $0x5,%rax
   c244a:	mov    %r15,(%r14,%rax,1)
   c244e:	mov    0x8(%rsp),%rcx
   c2453:	mov    %rcx,0x8(%r14,%rax,1)
   c2458:	mov    0x30(%rsp),%rcx
   c245d:	mov    %rcx,0x10(%r14,%rax,1)
   c2462:	mov    0x28(%rsp),%rcx
   c2467:	mov    %rcx,0x18(%r14,%rax,1)
   c246c:	inc    %rbx
   c246f:	mov    %rbx,0x78(%rsp)
   c2474:	movl   $0x2,0x18(%rsp)
   c247c:	movabs $0x8000000000000000,%r15
   c2486:	lea    -0xa1797(%rip),%r13        # 20cf6 <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.7926015133572591147+0x287>
   c248d:	lea    -0xa19a8(%rip),%r12        # 20aec <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.7926015133572591147+0x7d>
   c2494:	movzbl 0x7(%rsp),%ebp
   c2499:	lea    0xc8(%rsp),%rsi
   c24a1:	mov    0x38(%rsp),%rbx
   c24a6:	jmp    c1858 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x408>
   c24ab:	mov    $0x1,%r13d
   c24b1:	jmp    c24d0 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1080>
   c24b3:	mov    0x10(%rsp),%rax
   c24b8:	mov    $0x6,%r13d
   c24be:	mov    0x38(%rsp),%rbx
   c24c3:	jmp    c24d0 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1080>
   c24c5:	mov    0x10(%rsp),%rax
   c24ca:	mov    $0x6,%r13d
   c24d0:	mov    %rax,%r12
   c24d3:	movabs $0x8000000000000000,%r15
   c24dd:	mov    0x68(%rsp),%rsi
   c24e2:	test   %rsi,%rsi
   c24e5:	je     c24fb <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x10ab>
   c24e7:	mov    0x70(%rsp),%rdi
   c24ec:	shl    $0x5,%rsi
   c24f0:	mov    $0x8,%edx
   c24f5:	call   *0x1f53cd(%rip)        # 2b78c8 <_DYNAMIC+0x250>
   c24fb:	mov    0x80(%rsp),%rax
   c2503:	test   %rax,%rax
   c2506:	je     c251f <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x10cf>
   c2508:	mov    0x88(%rsp),%rdi
   c2510:	imul   $0x38,%rax,%rsi
   c2514:	mov    $0x8,%edx
   c2519:	call   *0x1f53a9(%rip)        # 2b78c8 <_DYNAMIC+0x250>
   c251f:	mov    %r13,0x8(%rbx)
   c2523:	mov    %r12,0x10(%rbx)
   c2527:	mov    %rbp,0x18(%rbx)
   c252b:	mov    %r14,0x20(%rbx)
   c252f:	mov    %r15,(%rbx)
   c2532:	mov    %rbx,%rax
   c2535:	add    $0x1f8,%rsp
   c253c:	pop    %rbx
   c253d:	pop    %r12
   c253f:	pop    %r13
   c2541:	pop    %r14
   c2543:	pop    %r15
   c2545:	pop    %rbp
   c2546:	ret
   c2547:	mov    $0xa,%ebp
   c254c:	mov    %r13,%rax
   c254f:	xor    %r13d,%r13d
   c2552:	jmp    c24d0 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1080>
   c2557:	mov    0x20(%rsp),%rax
   c255c:	jmp    c24ca <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x107a>
   c2561:	mov    %r9,%rax
   c2564:	mov    %r13,%r14
   c2567:	mov    %rdi,%r13
   c256a:	jmp    c24d0 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1080>
   c256f:	mov    $0x10,%ebp
   c2574:	lea    -0xaba4b(%rip),%rax        # 16b30 <__abi_tag+0x16834>
   c257b:	mov    $0x2,%r13d
   c2581:	jmp    c24d0 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1080>
   c2586:	mov    $0x7,%ebp
   c258b:	lea    -0xa1a77(%rip),%rax        # 20b1b <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.7926015133572591147+0xac>
   c2592:	xor    %r13d,%r13d
   c2595:	jmp    c24d0 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1080>
   c259a:	lea    -0xa18b8(%rip),%rax        # 20ce9 <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.7926015133572591147+0x27a>
   c25a1:	mov    $0xd,%ebp
   c25a6:	mov    $0x2,%r13d
   c25ac:	jmp    c24d0 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1080>
   c25b1:	mov    0x10(%rsp),%rax
   c25b6:	mov    $0x6,%r13d
   c25bc:	jmp    c25d0 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1180>
   c25be:	mov    $0x5,%r13d
   c25c4:	lea    -0xa19a0(%rip),%rax        # 20c2b <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.7926015133572591147+0x1bc>
   c25cb:	mov    $0xd,%ebp
   c25d0:	mov    %rbx,%r14
   c25d3:	mov    0x38(%rsp),%rbx
   c25d8:	jmp    c24d0 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1080>
   c25dd:	mov    $0x8,%r13d
   c25e3:	mov    $0xc,%ebp
   c25e8:	lea    -0xa19b7(%rip),%rax        # 20c38 <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.7926015133572591147+0x1c9>
   c25ef:	jmp    c24d0 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1080>
   c25f4:	movzbl %al,%eax
   c25f7:	cmp    $0x2,%eax
   c25fa:	jae    c275e <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x130e>
   c2600:	mov    $0x28,%ebp
   c2605:	lea    -0xa1996(%rip),%rax        # 20c76 <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.7926015133572591147+0x207>
   c260c:	mov    $0x8,%r13d
   c2612:	jmp    c24d0 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1080>
   c2617:	mov    %r9,%rax
   c261a:	jmp    c24ca <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x107a>
   c261f:	mov    $0xc,%ebp
   c2624:	lea    -0xa19f3(%rip),%rax        # 20c38 <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.7926015133572591147+0x1c9>
   c262b:	mov    $0x8,%r13d
   c2631:	jmp    c24d0 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1080>
   c2636:	mov    $0x1,%eax
   c263b:	jmp    c24b1 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1061>
   c2640:	mov    %r12,0x60(%rsp)
   c2645:	movabs $0x8000000000000000,%r15
   c264f:	lea    -0xa1b6a(%rip),%r12        # 20aec <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.7926015133572591147+0x7d>
   c2656:	mov    %ebp,%ecx
   c2658:	mov    $0x4,%r13d
   c265e:	mov    $0x7,%ebp
   c2663:	cmpb   $0x1,0xa0(%rsp)
   c266b:	jne    c2694 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1244>
   c266d:	testb  $0x1,0x54(%rsp)
   c2672:	je     c269c <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x124c>
   c2674:	cmpl   $0x1,0x50(%rsp)
   c2679:	jne    c26b0 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1260>
   c267b:	cmp    $0x5,%cl
   c267e:	jne    c26c4 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1274>
   c2680:	mov    $0xa,%ebp
   c2685:	xor    %r14d,%r14d
   c2688:	lea    -0xa1b7e(%rip),%r12        # 20b11 <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.7926015133572591147+0xa2>
   c268f:	jmp    c24dd <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x108d>
   c2694:	xor    %r14d,%r14d
   c2697:	jmp    c24dd <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x108d>
   c269c:	mov    $0x9,%ebp
   c26a1:	xor    %r14d,%r14d
   c26a4:	lea    -0xa1bb8(%rip),%r12        # 20af3 <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.7926015133572591147+0x84>
   c26ab:	jmp    c24dd <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x108d>
   c26b0:	mov    $0x3,%ebp
   c26b5:	xor    %r14d,%r14d
   c26b8:	lea    -0xa1bb1(%rip),%r12        # 20b0e <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.7926015133572591147+0x9f>
   c26bf:	jmp    c24dd <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x108d>
   c26c4:	mov    0x90(%rsp),%r12
   c26cc:	test   %r12,%r12
   c26cf:	je     c2775 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1325>
   c26d5:	mov    0x80(%rsp),%rax
   c26dd:	mov    0x88(%rsp),%r13
   c26e5:	mov    0x68(%rsp),%rbp
   c26ea:	cmp    %r15,%rax
   c26ed:	jne    c2784 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1334>
   c26f3:	mov    0x70(%rsp),%r14
   c26f8:	jmp    c251f <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x10cf>
   c26fd:	lea    -0xa19c4(%rip),%rax        # 20d40 <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.7926015133572591147+0x2d1>
   c2704:	mov    $0x2,%r13d
   c270a:	jmp    c24d0 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1080>
   c270f:	mov    $0x5,%r13d
   c2715:	mov    $0xb,%ebp
   c271a:	lea    -0xa1add(%rip),%rax        # 20c44 <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.7926015133572591147+0x1d5>
   c2721:	jmp    c24d0 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1080>
   c2726:	lea    -0xa1ade(%rip),%rax        # 20c4f <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.7926015133572591147+0x1e0>
   c272d:	mov    $0x8,%r13d
   c2733:	jmp    c24d0 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1080>
   c2738:	mov    $0x1a,%ebp
   c273d:	lea    -0xa1a1e(%rip),%rax        # 20d26 <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.7926015133572591147+0x2b7>
   c2744:	mov    $0x2,%r13d
   c274a:	jmp    c24d0 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1080>
   c274f:	lea    -0xa1a7b(%rip),%rax        # 20cdb <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.7926015133572591147+0x26c>
   c2756:	xor    %r13d,%r13d
   c2759:	jmp    c24d0 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1080>
   c275e:	mov    $0x4,%r13d
   c2764:	mov    $0x19,%ebp
   c2769:	lea    -0xa1b13(%rip),%rax        # 20c5d <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.7926015133572591147+0x1ee>
   c2770:	jmp    c24d0 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1080>
   c2775:	lea    -0xa1c61(%rip),%r12        # 20b1b <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.7926015133572591147+0xac>
   c277c:	xor    %r14d,%r14d
   c277f:	jmp    c24dd <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x108d>
   c2784:	movups 0x70(%rsp),%xmm0
   c2789:	mov    %rax,(%rbx)
   c278c:	mov    %r13,0x8(%rbx)
   c2790:	mov    %r12,0x10(%rbx)
   c2794:	mov    %rbp,0x18(%rbx)
   c2798:	movups %xmm0,0x20(%rbx)
   c279c:	mov    0x60(%rsp),%rax
   c27a1:	mov    %rax,0x30(%rbx)
   c27a5:	mov    0xac(%rsp),%eax
   c27ac:	mov    %eax,0x38(%rbx)
   c27af:	mov    %cl,0x3c(%rbx)
   c27b2:	jmp    c2532 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x10e2>
   c27b7:	lea    0x1e90ba(%rip),%rcx        # 2ab878 <anon.49e524b3d56d2aeb6c463ad9e106202e.63.llvm.7926015133572591147+0x30>
   c27be:	mov    $0x6,%edx
   c27c3:	xor    %edi,%edi
   c27c5:	mov    %r12,%rsi
   c27c8:	call   *0x1f5342(%rip)        # 2b7b10 <_DYNAMIC+0x498>
   c27ce:	ud2
   c27d0:	lea    -0xa1c69(%rip),%rax        # 20b6e <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.7926015133572591147+0xff>
   c27d7:	jmp    c24d0 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1080>
   c27dc:	lea    -0xa1ce7(%rip),%rdx        # 20afc <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.7926015133572591147+0x8d>
   c27e3:	mov    $0x12,%ecx
   c27e8:	mov    %rdi,%r15
   c27eb:	mov    %r9,%r12
   c27ee:	mov    %r9,%rsi
   c27f1:	call   c2aa0 <rvvdk_vmdk::descriptor::eq>
   c27f6:	test   %al,%al
   c27f8:	je     c282b <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x13db>
   c27fa:	lea    -0xa1cdf(%rip),%rax        # 20b22 <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.7926015133572591147+0xb3>
   c2801:	mov    $0xc,%ebp
   c2806:	mov    $0x5,%r13d
   c280c:	jmp    c24d0 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1080>
   c2811:	mov    $0x3,%r13d
   c2817:	jmp    c24b1 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1061>
   c281c:	mov    0x48(%rsp),%r14
   c2821:	mov    0x38(%rsp),%rbx
   c2826:	jmp    c24d0 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1080>
   c282b:	lea    -0xa1d46(%rip),%rdx        # 20aec <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.7926015133572591147+0x7d>
   c2832:	mov    $0x7,%ecx
   c2837:	mov    %r15,%rdi
   c283a:	mov    %r12,%rsi
   c283d:	call   c2aa0 <rvvdk_vmdk::descriptor::eq>
   c2842:	test   %al,%al
   c2844:	jne    c28b6 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1466>
   c2846:	lea    -0xa1d3f(%rip),%rdx        # 20b0e <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.7926015133572591147+0x9f>
   c284d:	mov    $0x3,%ecx
   c2852:	mov    %r15,%rdi
   c2855:	mov    %r12,%rsi
   c2858:	call   c2aa0 <rvvdk_vmdk::descriptor::eq>
   c285d:	test   %al,%al
   c285f:	jne    c28b6 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1466>
   c2861:	lea    -0xa1d75(%rip),%rdx        # 20af3 <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.7926015133572591147+0x84>
   c2868:	mov    $0x9,%ecx
   c286d:	mov    %r15,%rdi
   c2870:	mov    %r12,%rsi
   c2873:	call   c2aa0 <rvvdk_vmdk::descriptor::eq>
   c2878:	test   %al,%al
   c287a:	jne    c28b6 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1466>
   c287c:	lea    -0xa1d72(%rip),%rdx        # 20b11 <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.7926015133572591147+0xa2>
   c2883:	mov    $0xa,%ecx
   c2888:	mov    %r15,%rdi
   c288b:	mov    %r12,%rsi
   c288e:	call   c2aa0 <rvvdk_vmdk::descriptor::eq>
   c2893:	test   %al,%al
   c2895:	jne    c28b6 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1466>
   c2897:	lea    -0xab306(%rip),%rdx        # 17598 <anon.786f9ca1aa6bf68944dde0bec4ae4f7b.2.llvm.4745231054059156856+0x2d8>
   c289e:	mov    $0x8,%ecx
   c28a3:	mov    %r15,%rdi
   c28a6:	mov    %r12,%rsi
   c28a9:	call   c2aa0 <rvvdk_vmdk::descriptor::eq>
   c28ae:	test   %al,%al
   c28b0:	je     c29b0 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1560>
   c28b6:	lea    -0xa1d63(%rip),%rax        # 20b5a <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.7926015133572591147+0xeb>
   c28bd:	jmp    c24d0 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1080>
   c28c2:	mov    0x10(%rsp),%rax
   c28c7:	jmp    c24b1 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1061>
   c28cc:	mov    0x128(%rsp),%rax
   c28d4:	mov    0x130(%rsp),%rbp
   c28dc:	mov    0x138(%rsp),%r14
   c28e4:	jmp    c24d0 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1080>
   c28e9:	mov    $0x1a,%ebp
   c28ee:	lea    -0xa1c57(%rip),%rax        # 20c9e <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.7926015133572591147+0x22f>
   c28f5:	mov    $0x8,%r13d
   c28fb:	jmp    c24d0 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1080>
   c2900:	xor    $0x1,%al
   c2902:	movzbl %al,%eax
   c2905:	lea    0x3(,%rax,2),%r13
   c290d:	mov    $0x8,%ebp
   c2912:	lea    -0xab381(%rip),%rax        # 17598 <anon.786f9ca1aa6bf68944dde0bec4ae4f7b.2.llvm.4745231054059156856+0x2d8>
   c2919:	jmp    c24d0 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1080>
   c291e:	mov    $0x23,%ebp
   c2923:	lea    -0xa1c72(%rip),%rax        # 20cb8 <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.7926015133572591147+0x249>
   c292a:	mov    $0x8,%r13d
   c2930:	jmp    c24d0 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1080>
   c2935:	mov    $0x5,%r13d
   c293b:	mov    $0xc,%ebp
   c2940:	lea    -0xa1e25(%rip),%rax        # 20b22 <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.7926015133572591147+0xb3>
   c2947:	jmp    c24d0 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1080>
   c294c:	mov    $0x5,%r13d
   c2952:	mov    $0x12,%ebp
   c2957:	lea    -0xa1e30(%rip),%rax        # 20b2e <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.7926015133572591147+0xbf>
   c295e:	jmp    c24d0 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1080>
   c2963:	mov    $0x12,%ebp
   c2968:	lea    -0xa1ded(%rip),%rax        # 20b82 <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.7926015133572591147+0x113>
   c296f:	jmp    c281c <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x13cc>
   c2974:	mov    $0x18,%ebp
   c2979:	lea    -0xa1d75(%rip),%rax        # 20c0b <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.7926015133572591147+0x19c>
   c2980:	jmp    c281c <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x13cc>
   c2985:	mov    $0x5,%r13d
   c298b:	mov    $0xb,%ebp
   c2990:	lea    -0xa1e52(%rip),%rax        # 20b45 <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.7926015133572591147+0xd6>
   c2997:	jmp    c24d0 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1080>
   c299c:	mov    $0x10,%ebp
   c29a1:	lea    -0xab958(%rip),%rax        # 17050 <anon.83d0f48a2d0b5ff7b6e24226a84ebc25.25.llvm.569205349683904225+0x20>
   c29a8:	xor    %r13d,%r13d
   c29ab:	jmp    c281c <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x13cc>
   c29b0:	lea    -0xa1e67(%rip),%rax        # 20b50 <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.7926015133572591147+0xe1>
   c29b7:	mov    $0xa,%ebp
   c29bc:	mov    $0x5,%r13d
   c29c2:	jmp    c24d0 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1080>
   c29c7:	mov    %r9,%rax
   c29ca:	jmp    c24b1 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1061>
   c29cf:	mov    $0x5,%r13d
   c29d5:	mov    $0x7,%ebp
   c29da:	lea    -0xa1ddd(%rip),%rax        # 20c04 <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.7926015133572591147+0x195>
   c29e1:	jmp    c281c <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x13cc>
   c29e6:	jmp    c29ea <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x159a>
   c29e8:	jmp    c29ea <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x159a>
   c29ea:	mov    %rax,%rbx
   c29ed:	mov    0x68(%rsp),%rsi
   c29f2:	test   %rsi,%rsi
   c29f5:	jne    c2a0c <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x15bc>
   c29f7:	mov    0x80(%rsp),%rax
   c29ff:	test   %rax,%rax
   c2a02:	jne    c2a2d <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x15dd>
   c2a04:	mov    %rbx,%rdi
   c2a07:	call   2a94e0 <_Unwind_Resume@plt>
   c2a0c:	mov    0x70(%rsp),%rdi
   c2a11:	shl    $0x5,%rsi
   c2a15:	mov    $0x8,%edx
   c2a1a:	call   *0x1f4ea8(%rip)        # 2b78c8 <_DYNAMIC+0x250>
   c2a20:	mov    0x80(%rsp),%rax
   c2a28:	test   %rax,%rax
   c2a2b:	je     c2a04 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x15b4>
   c2a2d:	mov    0x88(%rsp),%rdi
   c2a35:	imul   $0x38,%rax,%rsi
   c2a39:	mov    $0x8,%edx
   c2a3e:	call   *0x1f4e84(%rip)        # 2b78c8 <_DYNAMIC+0x250>
   c2a44:	mov    %rbx,%rdi
   c2a47:	call   2a94e0 <_Unwind_Resume@plt>
