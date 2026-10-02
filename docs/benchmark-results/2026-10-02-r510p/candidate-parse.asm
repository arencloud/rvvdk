
target/r510p/reference/descriptor-candidate:     file format elf64-x86-64


Disassembly of section .text:

00000000000c14d0 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits>:
   c14d0:	push   %rbp
   c14d1:	push   %r15
   c14d3:	push   %r14
   c14d5:	push   %r13
   c14d7:	push   %r12
   c14d9:	push   %rbx
   c14da:	sub    $0x1c8,%rsp
   c14e1:	mov    %rdi,%rbx
   c14e4:	movabs $0x8000000000000000,%r15
   c14ee:	cmp    (%rcx),%rdx
   c14f1:	jbe    c150a <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x3a>
   c14f3:	lea    -0xaa46a(%rip),%rbp        # 17090 <anon.83d0f48a2d0b5ff7b6e24226a84ebc25.25.llvm.569205349683904225+0x60>
   c14fa:	mov    $0x10,%r12d
   c1500:	xor    %ecx,%ecx
   c1502:	xor    %r13d,%r13d
   c1505:	jmp    c2d14 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1844>
   c150a:	mov    %rcx,%r14
   c150d:	lea    0x110(%rsp),%rdi
   c1515:	call   *0x1f760d(%rip)        # 2b8b28 <_DYNAMIC+0x7e0>
   c151b:	cmpl   $0x1,0x110(%rsp)
   c1523:	jne    c1532 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x62>
   c1525:	mov    $0x1,%ecx
   c152a:	xor    %r13d,%r13d
   c152d:	jmp    c2d14 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1844>
   c1532:	mov    0x118(%rsp),%rax
   c153a:	mov    0x120(%rsp),%rcx
   c1542:	movq   $0x0,0x78(%rsp)
   c154b:	movq   $0x8,0x80(%rsp)
   c1557:	movq   $0x0,0x88(%rsp)
   c1563:	movq   $0x0,0x50(%rsp)
   c156c:	movq   $0x8,0x58(%rsp)
   c1575:	movq   $0x0,0x60(%rsp)
   c157e:	xorps  %xmm0,%xmm0
   c1581:	movaps %xmm0,0xc0(%rsp)
   c1589:	mov    %rcx,0xd0(%rsp)
   c1591:	mov    %rax,0xd8(%rsp)
   c1599:	mov    %rcx,0xe0(%rsp)
   c15a1:	movq   $0x0,0xe8(%rsp)
   c15ad:	mov    %rcx,0xf0(%rsp)
   c15b5:	movabs $0xa0000000a,%rax
   c15bf:	mov    %rax,0xf8(%rsp)
   c15c7:	movb   $0x1,0x100(%rsp)
   c15cf:	movw   $0x1,0x108(%rsp)
   c15d9:	mov    0x8(%r14),%rax
   c15dd:	mov    %rax,0x1c0(%rsp)
   c15e5:	mov    0x10(%r14),%rax
   c15e9:	mov    %rax,0x1b8(%rsp)
   c15f1:	mov    0x18(%r14),%rax
   c15f5:	mov    %rax,0x1b0(%rsp)
   c15fd:	mov    0x20(%r14),%rax
   c1601:	mov    %rax,0x1a8(%rsp)
   c1609:	movb   $0x5,0x17(%rsp)
   c160e:	lea    -0xa08b3(%rip),%rax        # 20d62 <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.12047894789963178251+0x303>
   c1615:	mov    %rax,0x8(%rsp)
   c161a:	movl   $0x0,0x40(%rsp)
   c1622:	movl   $0x0,0x44(%rsp)
   c162a:	movq   $0x0,0xa0(%rsp)
   c1636:	movq   $0x0,0xb8(%rsp)
   c1642:	xor    %ebp,%ebp
   c1644:	xor    %r14d,%r14d
   c1647:	mov    %r14,0x70(%rsp)
   c164c:	lea    0x110(%rsp),%r14
   c1654:	mov    0xd8(%rsp),%r15
   c165c:	mov    %r14,%rdi
   c165f:	lea    0xd8(%rsp),%rsi
   c1667:	call   c12c0 <<core::str::pattern::CharSearcher as core::str::pattern::Searcher>::next_match>
   c166c:	cmpl   $0x1,0x110(%rsp)
   c1674:	jne    c169e <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1ce>
   c1676:	mov    0x118(%rsp),%rax
   c167e:	mov    0x120(%rsp),%rcx
   c1686:	mov    0xc8(%rsp),%rdx
   c168e:	sub    %rdx,%rax
   c1691:	add    %rdx,%r15
   c1694:	mov    %rcx,0xc8(%rsp)
   c169c:	jmp    c16e2 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x212>
   c169e:	cmpb   $0x0,0x109(%rsp)
   c16a6:	jne    c2d3c <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x186c>
   c16ac:	movb   $0x1,0x109(%rsp)
   c16b4:	mov    0xc8(%rsp),%r15
   c16bc:	mov    0xd0(%rsp),%rax
   c16c4:	sub    %r15,%rax
   c16c7:	setne  %cl
   c16ca:	or     0x108(%rsp),%cl
   c16d1:	cmp    $0x1,%cl
   c16d4:	jne    c2d3c <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x186c>
   c16da:	add    0xd8(%rsp),%r15
   c16e2:	mov    0xc0(%rsp),%r13
   c16ea:	inc    %r13
   c16ed:	mov    %r13,0xc0(%rsp)
   c16f5:	mov    $0xa,%r12d
   c16fb:	cmp    0x1c0(%rsp),%rax
   c1703:	ja     c2d93 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x18c3>
   c1709:	xor    %edx,%edx
   c170b:	test   %rax,%rax
   c170e:	je     c171c <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x24c>
   c1710:	cmpb   $0xd,-0x1(%r15,%rax,1)
   c1716:	sete   %dl
   c1719:	neg    %rdx
   c171c:	add    %rax,%rdx
   c171f:	lea    (%r15,%rdx,1),%rax
   c1723:	mov    %r15,%rcx
   c1726:	jmp    c1759 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x289>
   c1728:	nopl   0x0(%rax,%rax,1)
   c1730:	inc    %rcx
   c1733:	lea    -0x7f(%rsi),%edi
   c1736:	xor    %r8d,%r8d
   c1739:	cmp    $0x21,%edi
   c173c:	setb   %r8b
   c1740:	xor    %edi,%edi
   c1742:	cmp    $0x9,%esi
   c1745:	setne  %dil
   c1749:	cmp    $0x20,%esi
   c174c:	cmovb  %edi,%r8d
   c1750:	test   %r8b,%r8b
   c1753:	jne    c2cba <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x17ea>
   c1759:	cmp    %rax,%rcx
   c175c:	je     c17da <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x30a>
   c175e:	movzbl (%rcx),%esi
   c1761:	test   %sil,%sil
   c1764:	jns    c1730 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x260>
   c1766:	mov    %esi,%edi
   c1768:	and    $0x1f,%edi
   c176b:	movzbl 0x1(%rcx),%r9d
   c1770:	and    $0x3f,%r9d
   c1774:	cmp    $0xdf,%sil
   c1778:	jbe    c17b7 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x2e7>
   c177a:	movzbl 0x2(%rcx),%r8d
   c177f:	shl    $0x6,%r9d
   c1783:	and    $0x3f,%r8d
   c1787:	or     %r9d,%r8d
   c178a:	cmp    $0xf0,%sil
   c178e:	jb     c17c8 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x2f8>
   c1790:	movzbl 0x3(%rcx),%esi
   c1794:	and    $0x7,%edi
   c1797:	shl    $0x12,%edi
   c179a:	shl    $0x6,%r8d
   c179e:	and    $0x3f,%esi
   c17a1:	or     %r8d,%esi
   c17a4:	or     %edi,%esi
   c17a6:	cmp    $0x110000,%esi
   c17ac:	je     c17da <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x30a>
   c17ae:	add    $0x4,%rcx
   c17b2:	jmp    c1733 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x263>
   c17b7:	add    $0x2,%rcx
   c17bb:	shl    $0x6,%edi
   c17be:	or     %r9d,%edi
   c17c1:	mov    %edi,%esi
   c17c3:	jmp    c1733 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x263>
   c17c8:	add    $0x3,%rcx
   c17cc:	shl    $0xc,%edi
   c17cf:	or     %edi,%r8d
   c17d2:	mov    %r8d,%esi
   c17d5:	jmp    c1733 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x263>
   c17da:	mov    %r14,%rdi
   c17dd:	mov    %r15,%rsi
   c17e0:	mov    %r13,%rcx
   c17e3:	call   c38e0 <rvvdk_vmdk::descriptor::tokens>
   c17e8:	mov    0x110(%rsp),%rcx
   c17f0:	mov    0x118(%rsp),%rax
   c17f8:	mov    %rax,0x18(%rsp)
   c17fd:	mov    0x120(%rsp),%r15
   c1805:	mov    0x128(%rsp),%rdx
   c180d:	mov    0x130(%rsp),%r14
   c1815:	cmp    $0x3,%rcx
   c1819:	je     c2db4 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x18e4>
   c181f:	mov    0x1a0(%rsp),%rsi
   c1827:	cmp    $0x7,%rsi
   c182b:	jae    c303e <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1b6e>
   c1831:	mov    0x138(%rsp),%rax
   c1839:	mov    0x140(%rsp),%rdi
   c1841:	mov    %rdi,0x48(%rsp)
   c1846:	mov    0x148(%rsp),%rdi
   c184e:	mov    %rdi,0x38(%rsp)
   c1853:	mov    0x150(%rsp),%rdi
   c185b:	mov    %rdi,0x28(%rsp)
   c1860:	mov    0x158(%rsp),%r10
   c1868:	mov    0x160(%rsp),%rdi
   c1870:	mov    0x168(%rsp),%r8
   c1878:	mov    %r8,0x68(%rsp)
   c187d:	mov    0x170(%rsp),%r11
   c1885:	mov    0x178(%rsp),%r8
   c188d:	mov    0x180(%rsp),%r9
   c1895:	cmp    $0x3,%rsi
   c1899:	je     c18bf <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x3ef>
   c189b:	test   %rsi,%rsi
   c189e:	jne    c26dc <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x120c>
   c18a4:	lea    0x110(%rsp),%r14
   c18ac:	cmpb   $0x0,0x109(%rsp)
   c18b4:	je     c1654 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x184>
   c18ba:	jmp    c2d3c <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x186c>
   c18bf:	test   %rcx,%rcx
   c18c2:	jne    c26dc <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x120c>
   c18c8:	cmp    $0x2,%rdx
   c18cc:	jne    c26dc <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x120c>
   c18d2:	cmp    $0x4,%r15
   c18d6:	jbe    c18e8 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x418>
   c18d8:	mov    0x18(%rsp),%rax
   c18dd:	cmpb   $0xc0,0x4(%rax)
   c18e1:	jge    c18ee <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x41e>
   c18e3:	jmp    c19bc <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x4ec>
   c18e8:	jne    c19bc <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x4ec>
   c18ee:	mov    0x18(%rsp),%rax
   c18f3:	movzbl (%rax),%eax
   c18f6:	lea    -0x41(%rax),%ecx
   c18f9:	cmp    $0x1a,%cl
   c18fc:	setb   %cl
   c18ff:	shl    $0x5,%cl
   c1902:	or     %al,%cl
   c1904:	cmp    $0x64,%cl
   c1907:	jne    c19bc <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x4ec>
   c190d:	mov    0x18(%rsp),%rax
   c1912:	movzbl 0x1(%rax),%eax
   c1916:	lea    -0x41(%rax),%ecx
   c1919:	cmp    $0x1a,%cl
   c191c:	setb   %cl
   c191f:	shl    $0x5,%cl
   c1922:	or     %al,%cl
   c1924:	cmp    $0x64,%cl
   c1927:	jne    c19bc <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x4ec>
   c192d:	mov    0x18(%rsp),%rax
   c1932:	movzbl 0x2(%rax),%eax
   c1936:	lea    -0x41(%rax),%ecx
   c1939:	cmp    $0x1a,%cl
   c193c:	setb   %cl
   c193f:	shl    $0x5,%cl
   c1942:	or     %al,%cl
   c1944:	cmp    $0x62,%cl
   c1947:	jne    c19bc <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x4ec>
   c1949:	mov    0x18(%rsp),%rax
   c194e:	movzbl 0x3(%rax),%eax
   c1952:	lea    -0x41(%rax),%ecx
   c1955:	cmp    $0x1a,%cl
   c1958:	setb   %cl
   c195b:	shl    $0x5,%cl
   c195e:	or     %al,%cl
   c1960:	cmp    $0x2e,%cl
   c1963:	jne    c19bc <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x4ec>
   c1965:	mov    $0x2,%r8d
   c196b:	test   %ebp,%ebp
   c196d:	je     c3493 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1fc3>
   c1973:	cmp    $0xf,%r15
   c1977:	mov    0x18(%rsp),%rdi
   c197c:	mov    $0x7,%r12d
   c1982:	jbe    c237f <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0xeaf>
   c1988:	lea    -0x10(%r15),%rax
   c198c:	cmp    $0x6,%rax
   c1990:	ja     c2fa0 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1ad0>
   c1996:	lea    -0xa0e45(%rip),%rcx        # 20b58 <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.12047894789963178251+0xf9>
   c199d:	movslq (%rcx,%rax,4),%rax
   c19a1:	add    %rcx,%rax
   c19a4:	jmp    *%rax
   c19a6:	mov    $0x10,%esi
   c19ab:	mov    $0x10,%ecx
   c19b0:	lea    -0xaa687(%rip),%rdx        # 17330 <anon.786f9ca1aa6bf68944dde0bec4ae4f7b.2.llvm.4745231054059156856+0x70>
   c19b7:	jmp    c2598 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x10c8>
   c19bc:	mov    $0x2,%r8d
   c19c2:	test   %ebp,%ebp
   c19c4:	jne    c2df3 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1923>
   c19ca:	mov    0x48(%rsp),%rax
   c19cf:	test   %rax,%rax
   c19d2:	je     c1d02 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x832>
   c19d8:	cmp    $0x1,%rax
   c19dc:	lea    0x110(%rsp),%r14
   c19e4:	mov    0x18(%rsp),%rdi
   c19e9:	jne    c2e0a <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x193a>
   c19ef:	cmp    $0x8,%r15
   c19f3:	je     c1dd6 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x906>
   c19f9:	cmp    $0xa,%r15
   c19fd:	jne    c2e0a <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x193a>
   c1a03:	movzbl (%rdi),%eax
   c1a06:	lea    -0x41(%rax),%ecx
   c1a09:	cmp    $0x1a,%cl
   c1a0c:	setb   %dl
   c1a0f:	shl    $0x5,%dl
   c1a12:	or     %al,%dl
   c1a14:	cmp    $0x63,%dl
   c1a17:	jne    c337b <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1eab>
   c1a1d:	movzbl 0x1(%rdi),%edx
   c1a21:	lea    -0x41(%rdx),%esi
   c1a24:	cmp    $0x1a,%sil
   c1a28:	setb   %sil
   c1a2c:	shl    $0x5,%sil
   c1a30:	or     %dl,%sil
   c1a33:	cmp    $0x72,%sil
   c1a37:	jne    c337b <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1eab>
   c1a3d:	movzbl 0x2(%rdi),%edx
   c1a41:	lea    -0x41(%rdx),%esi
   c1a44:	cmp    $0x1a,%sil
   c1a48:	setb   %sil
   c1a4c:	shl    $0x5,%sil
   c1a50:	or     %dl,%sil
   c1a53:	cmp    $0x65,%sil
   c1a57:	jne    c337b <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1eab>
   c1a5d:	movzbl 0x3(%rdi),%edx
   c1a61:	lea    -0x41(%rdx),%esi
   c1a64:	cmp    $0x1a,%sil
   c1a68:	setb   %sil
   c1a6c:	shl    $0x5,%sil
   c1a70:	or     %dl,%sil
   c1a73:	cmp    $0x61,%sil
   c1a77:	jne    c337b <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1eab>
   c1a7d:	movzbl 0x4(%rdi),%edx
   c1a81:	lea    -0x41(%rdx),%esi
   c1a84:	cmp    $0x1a,%sil
   c1a88:	setb   %sil
   c1a8c:	shl    $0x5,%sil
   c1a90:	or     %dl,%sil
   c1a93:	cmp    $0x74,%sil
   c1a97:	jne    c337b <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1eab>
   c1a9d:	movzbl 0x5(%rdi),%edx
   c1aa1:	lea    -0x41(%rdx),%esi
   c1aa4:	cmp    $0x1a,%sil
   c1aa8:	setb   %sil
   c1aac:	shl    $0x5,%sil
   c1ab0:	or     %dl,%sil
   c1ab3:	cmp    $0x65,%sil
   c1ab7:	jne    c337b <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1eab>
   c1abd:	movzbl 0x6(%rdi),%edx
   c1ac1:	lea    -0x41(%rdx),%esi
   c1ac4:	cmp    $0x1a,%sil
   c1ac8:	setb   %sil
   c1acc:	shl    $0x5,%sil
   c1ad0:	or     %dl,%sil
   c1ad3:	cmp    $0x74,%sil
   c1ad7:	jne    c337b <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1eab>
   c1add:	movzbl 0x7(%rdi),%edx
   c1ae1:	lea    -0x41(%rdx),%esi
   c1ae4:	cmp    $0x1a,%sil
   c1ae8:	setb   %sil
   c1aec:	shl    $0x5,%sil
   c1af0:	or     %dl,%sil
   c1af3:	cmp    $0x79,%sil
   c1af7:	jne    c337b <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1eab>
   c1afd:	movzbl 0x8(%rdi),%edx
   c1b01:	lea    -0x41(%rdx),%esi
   c1b04:	cmp    $0x1a,%sil
   c1b08:	setb   %sil
   c1b0c:	shl    $0x5,%sil
   c1b10:	or     %dl,%sil
   c1b13:	cmp    $0x70,%sil
   c1b17:	jne    c337b <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1eab>
   c1b1d:	movzbl 0x9(%rdi),%edx
   c1b21:	lea    -0x41(%rdx),%esi
   c1b24:	cmp    $0x1a,%sil
   c1b28:	setb   %sil
   c1b2c:	shl    $0x5,%sil
   c1b30:	or     %dl,%sil
   c1b33:	cmp    $0x65,%sil
   c1b37:	jne    c337b <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1eab>
   c1b3d:	mov    0x28(%rsp),%rax
   c1b42:	add    $0xfffffffffffffffa,%rax
   c1b46:	ror    $1,%rax
   c1b49:	mov    $0x5,%r8d
   c1b4f:	mov    $0xb,%r12d
   c1b55:	cmp    $0x6,%rax
   c1b59:	ja     c2e75 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x19a5>
   c1b5f:	lea    -0xa108a(%rip),%rcx        # 20adc <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.12047894789963178251+0x7d>
   c1b66:	movslq (%rcx,%rax,4),%rax
   c1b6a:	add    %rcx,%rax
   c1b6d:	mov    0x38(%rsp),%r15
   c1b72:	jmp    *%rax
   c1b74:	movzbl (%r15),%eax
   c1b78:	lea    -0x41(%rax),%ecx
   c1b7b:	cmp    $0x1a,%cl
   c1b7e:	setb   %cl
   c1b81:	shl    $0x5,%cl
   c1b84:	or     %al,%cl
   c1b86:	cmp    $0x6d,%cl
   c1b89:	jne    c222e <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0xd5e>
   c1b8f:	movzbl 0x1(%r15),%eax
   c1b94:	lea    -0x41(%rax),%ecx
   c1b97:	cmp    $0x1a,%cl
   c1b9a:	setb   %cl
   c1b9d:	shl    $0x5,%cl
   c1ba0:	or     %al,%cl
   c1ba2:	cmp    $0x6f,%cl
   c1ba5:	jne    c222e <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0xd5e>
   c1bab:	movzbl 0x2(%r15),%eax
   c1bb0:	lea    -0x41(%rax),%ecx
   c1bb3:	cmp    $0x1a,%cl
   c1bb6:	setb   %cl
   c1bb9:	shl    $0x5,%cl
   c1bbc:	or     %al,%cl
   c1bbe:	cmp    $0x6e,%cl
   c1bc1:	jne    c222e <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0xd5e>
   c1bc7:	movzbl 0x3(%r15),%eax
   c1bcc:	lea    -0x41(%rax),%ecx
   c1bcf:	cmp    $0x1a,%cl
   c1bd2:	setb   %cl
   c1bd5:	shl    $0x5,%cl
   c1bd8:	or     %al,%cl
   c1bda:	cmp    $0x6f,%cl
   c1bdd:	jne    c222e <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0xd5e>
   c1be3:	movzbl 0x4(%r15),%eax
   c1be8:	lea    -0x41(%rax),%ecx
   c1beb:	cmp    $0x1a,%cl
   c1bee:	setb   %cl
   c1bf1:	shl    $0x5,%cl
   c1bf4:	or     %al,%cl
   c1bf6:	cmp    $0x6c,%cl
   c1bf9:	jne    c222e <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0xd5e>
   c1bff:	movzbl 0x5(%r15),%eax
   c1c04:	lea    -0x41(%rax),%ecx
   c1c07:	cmp    $0x1a,%cl
   c1c0a:	setb   %cl
   c1c0d:	shl    $0x5,%cl
   c1c10:	or     %al,%cl
   c1c12:	cmp    $0x69,%cl
   c1c15:	jne    c222e <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0xd5e>
   c1c1b:	movzbl 0x6(%r15),%eax
   c1c20:	lea    -0x41(%rax),%ecx
   c1c23:	cmp    $0x1a,%cl
   c1c26:	setb   %cl
   c1c29:	shl    $0x5,%cl
   c1c2c:	or     %al,%cl
   c1c2e:	cmp    $0x74,%cl
   c1c31:	jne    c222e <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0xd5e>
   c1c37:	movzbl 0x7(%r15),%eax
   c1c3c:	lea    -0x41(%rax),%ecx
   c1c3f:	cmp    $0x1a,%cl
   c1c42:	setb   %cl
   c1c45:	shl    $0x5,%cl
   c1c48:	or     %al,%cl
   c1c4a:	cmp    $0x68,%cl
   c1c4d:	jne    c222e <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0xd5e>
   c1c53:	movzbl 0x8(%r15),%eax
   c1c58:	lea    -0x41(%rax),%ecx
   c1c5b:	cmp    $0x1a,%cl
   c1c5e:	setb   %cl
   c1c61:	shl    $0x5,%cl
   c1c64:	or     %al,%cl
   c1c66:	cmp    $0x69,%cl
   c1c69:	jne    c222e <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0xd5e>
   c1c6f:	movzbl 0x9(%r15),%eax
   c1c74:	lea    -0x41(%rax),%ecx
   c1c77:	cmp    $0x1a,%cl
   c1c7a:	setb   %cl
   c1c7d:	shl    $0x5,%cl
   c1c80:	or     %al,%cl
   c1c82:	cmp    $0x63,%cl
   c1c85:	jne    c222e <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0xd5e>
   c1c8b:	movzbl 0xa(%r15),%eax
   c1c90:	lea    -0x41(%rax),%ecx
   c1c93:	cmp    $0x1a,%cl
   c1c96:	setb   %cl
   c1c99:	shl    $0x5,%cl
   c1c9c:	or     %al,%cl
   c1c9e:	cmp    $0x66,%cl
   c1ca1:	jne    c222e <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0xd5e>
   c1ca7:	movzbl 0xb(%r15),%eax
   c1cac:	lea    -0x41(%rax),%ecx
   c1caf:	cmp    $0x1a,%cl
   c1cb2:	setb   %cl
   c1cb5:	shl    $0x5,%cl
   c1cb8:	or     %al,%cl
   c1cba:	cmp    $0x6c,%cl
   c1cbd:	jne    c222e <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0xd5e>
   c1cc3:	movzbl 0xc(%r15),%eax
   c1cc8:	lea    -0x41(%rax),%ecx
   c1ccb:	cmp    $0x1a,%cl
   c1cce:	setb   %cl
   c1cd1:	shl    $0x5,%cl
   c1cd4:	or     %al,%cl
   c1cd6:	cmp    $0x61,%cl
   c1cd9:	jne    c222e <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0xd5e>
   c1cdf:	movzbl 0xd(%r15),%eax
   c1ce4:	lea    -0x41(%rax),%ecx
   c1ce7:	cmp    $0x1a,%cl
   c1cea:	setb   %cl
   c1ced:	shl    $0x5,%cl
   c1cf0:	or     %al,%cl
   c1cf2:	cmp    $0x74,%cl
   c1cf5:	jne    c222e <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0xd5e>
   c1cfb:	xor    %ecx,%ecx
   c1cfd:	jmp    c2369 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0xe99>
   c1d02:	lea    -0x3(%r15),%rax
   c1d06:	cmp    $0xf,%rax
   c1d0a:	lea    0x110(%rsp),%r14
   c1d12:	mov    0x18(%rsp),%rdi
   c1d17:	ja     c2e86 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x19b6>
   c1d1d:	lea    -0xa122c(%rip),%rcx        # 20af8 <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.12047894789963178251+0x99>
   c1d24:	movslq (%rcx,%rax,4),%rax
   c1d28:	add    %rcx,%rax
   c1d2b:	jmp    *%rax
   c1d2d:	movzbl (%rdi),%eax
   c1d30:	lea    -0x41(%rax),%ecx
   c1d33:	cmp    $0x1a,%cl
   c1d36:	setb   %dl
   c1d39:	shl    $0x5,%dl
   c1d3c:	or     %al,%dl
   c1d3e:	cmp    $0x63,%dl
   c1d41:	jne    c2eb3 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x19e3>
   c1d47:	movzbl 0x1(%rdi),%edx
   c1d4b:	lea    -0x41(%rdx),%esi
   c1d4e:	cmp    $0x1a,%sil
   c1d52:	setb   %sil
   c1d56:	shl    $0x5,%sil
   c1d5a:	or     %dl,%sil
   c1d5d:	cmp    $0x69,%sil
   c1d61:	jne    c2eb3 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x19e3>
   c1d67:	movzbl 0x2(%rdi),%edx
   c1d6b:	lea    -0x41(%rdx),%esi
   c1d6e:	cmp    $0x1a,%sil
   c1d72:	setb   %sil
   c1d76:	shl    $0x5,%sil
   c1d7a:	or     %dl,%sil
   c1d7d:	cmp    $0x64,%sil
   c1d81:	jne    c2eb3 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x19e3>
   c1d87:	mov    %r14,%rdi
   c1d8a:	mov    0x38(%rsp),%rsi
   c1d8f:	mov    0x28(%rsp),%rdx
   c1d94:	mov    %r13,%rcx
   c1d97:	call   c3690 <rvvdk_vmdk::descriptor::cid>
   c1d9c:	mov    0x110(%rsp),%r8
   c1da4:	cmp    $0x9,%r8
   c1da8:	jne    c2f4f <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1a7f>
   c1dae:	cmpl   $0x1,0x44(%rsp)
   c1db3:	je     c2e55 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1985>
   c1db9:	mov    0x118(%rsp),%eax
   c1dc0:	mov    %eax,0xac(%rsp)
   c1dc7:	movl   $0x1,0x44(%rsp)
   c1dcf:	xor    %ebp,%ebp
   c1dd1:	jmp    c18ac <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x3dc>
   c1dd6:	movzbl (%rdi),%eax
   c1dd9:	lea    -0x41(%rax),%ecx
   c1ddc:	cmp    $0x1a,%cl
   c1ddf:	setb   %dl
   c1de2:	shl    $0x5,%dl
   c1de5:	or     %al,%dl
   c1de7:	cmp    $0x65,%dl
   c1dea:	jne    c309c <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1bcc>
   c1df0:	movzbl 0x1(%rdi),%edx
   c1df4:	lea    -0x41(%rdx),%esi
   c1df7:	cmp    $0x1a,%sil
   c1dfb:	setb   %sil
   c1dff:	shl    $0x5,%sil
   c1e03:	or     %dl,%sil
   c1e06:	cmp    $0x6e,%sil
   c1e0a:	jne    c309c <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1bcc>
   c1e10:	movzbl 0x2(%rdi),%edx
   c1e14:	lea    -0x41(%rdx),%esi
   c1e17:	cmp    $0x1a,%sil
   c1e1b:	setb   %sil
   c1e1f:	shl    $0x5,%sil
   c1e23:	or     %dl,%sil
   c1e26:	cmp    $0x63,%sil
   c1e2a:	jne    c309c <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1bcc>
   c1e30:	movzbl 0x3(%rdi),%edx
   c1e34:	lea    -0x41(%rdx),%esi
   c1e37:	cmp    $0x1a,%sil
   c1e3b:	setb   %sil
   c1e3f:	shl    $0x5,%sil
   c1e43:	or     %dl,%sil
   c1e46:	cmp    $0x6f,%sil
   c1e4a:	jne    c309c <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1bcc>
   c1e50:	movzbl 0x4(%rdi),%edx
   c1e54:	lea    -0x41(%rdx),%esi
   c1e57:	cmp    $0x1a,%sil
   c1e5b:	setb   %sil
   c1e5f:	shl    $0x5,%sil
   c1e63:	or     %dl,%sil
   c1e66:	cmp    $0x64,%sil
   c1e6a:	jne    c309c <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1bcc>
   c1e70:	movzbl 0x5(%rdi),%edx
   c1e74:	lea    -0x41(%rdx),%esi
   c1e77:	cmp    $0x1a,%sil
   c1e7b:	setb   %sil
   c1e7f:	shl    $0x5,%sil
   c1e83:	or     %dl,%sil
   c1e86:	cmp    $0x69,%sil
   c1e8a:	jne    c309c <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1bcc>
   c1e90:	movzbl 0x6(%rdi),%edx
   c1e94:	lea    -0x41(%rdx),%esi
   c1e97:	cmp    $0x1a,%sil
   c1e9b:	setb   %sil
   c1e9f:	shl    $0x5,%sil
   c1ea3:	or     %dl,%sil
   c1ea6:	cmp    $0x6e,%sil
   c1eaa:	jne    c309c <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1bcc>
   c1eb0:	movzbl 0x7(%rdi),%edx
   c1eb4:	lea    -0x41(%rdx),%esi
   c1eb7:	cmp    $0x1a,%sil
   c1ebb:	setb   %sil
   c1ebf:	shl    $0x5,%sil
   c1ec3:	or     %dl,%sil
   c1ec6:	cmp    $0x67,%sil
   c1eca:	jne    c309c <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1bcc>
   c1ed0:	mov    $0x5,%r8d
   c1ed6:	mov    $0x8,%r12d
   c1edc:	cmpq   $0x5,0x28(%rsp)
   c1ee2:	jne    c2f80 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1ab0>
   c1ee8:	mov    0x38(%rsp),%rdx
   c1eed:	movzbl (%rdx),%eax
   c1ef0:	lea    -0x41(%rax),%ecx
   c1ef3:	cmp    $0x1a,%cl
   c1ef6:	setb   %cl
   c1ef9:	shl    $0x5,%cl
   c1efc:	or     %al,%cl
   c1efe:	cmp    $0x75,%cl
   c1f01:	jne    c2f80 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1ab0>
   c1f07:	movzbl 0x1(%rdx),%eax
   c1f0b:	lea    -0x41(%rax),%ecx
   c1f0e:	cmp    $0x1a,%cl
   c1f11:	setb   %cl
   c1f14:	shl    $0x5,%cl
   c1f17:	or     %al,%cl
   c1f19:	cmp    $0x74,%cl
   c1f1c:	jne    c2f80 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1ab0>
   c1f22:	movzbl 0x2(%rdx),%eax
   c1f26:	lea    -0x41(%rax),%ecx
   c1f29:	cmp    $0x1a,%cl
   c1f2c:	setb   %cl
   c1f2f:	shl    $0x5,%cl
   c1f32:	or     %al,%cl
   c1f34:	cmp    $0x66,%cl
   c1f37:	jne    c2f80 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1ab0>
   c1f3d:	movzbl 0x3(%rdx),%eax
   c1f41:	lea    -0x41(%rax),%ecx
   c1f44:	cmp    $0x1a,%cl
   c1f47:	setb   %cl
   c1f4a:	shl    $0x5,%cl
   c1f4d:	or     %al,%cl
   c1f4f:	cmp    $0x2d,%cl
   c1f52:	jne    c2f80 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1ab0>
   c1f58:	movzbl 0x4(%rdx),%ecx
   c1f5c:	lea    -0x41(%rcx),%eax
   c1f5f:	cmp    $0x1a,%al
   c1f61:	setb   %al
   c1f64:	shl    $0x5,%al
   c1f67:	or     %cl,%al
   c1f69:	cmp    $0x38,%al
   c1f6b:	setne  %cl
   c1f6e:	or     0xb8(%rsp),%cl
   c1f75:	test   $0x1,%cl
   c1f78:	jne    c2f71 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1aa1>
   c1f7e:	mov    $0x1,%al
   c1f80:	mov    %rax,0xb8(%rsp)
   c1f88:	xor    %ebp,%ebp
   c1f8a:	jmp    c18ac <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x3dc>
   c1f8f:	movzbl (%rdi),%eax
   c1f92:	lea    -0x41(%rax),%ecx
   c1f95:	cmp    $0x1a,%cl
   c1f98:	setb   %dl
   c1f9b:	shl    $0x5,%dl
   c1f9e:	or     %al,%dl
   c1fa0:	cmp    $0x76,%dl
   c1fa3:	jne    c3189 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1cb9>
   c1fa9:	movzbl 0x1(%rdi),%edx
   c1fad:	lea    -0x41(%rdx),%esi
   c1fb0:	cmp    $0x1a,%sil
   c1fb4:	setb   %sil
   c1fb8:	shl    $0x5,%sil
   c1fbc:	or     %dl,%sil
   c1fbf:	cmp    $0x65,%sil
   c1fc3:	jne    c3189 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1cb9>
   c1fc9:	movzbl 0x2(%rdi),%edx
   c1fcd:	lea    -0x41(%rdx),%esi
   c1fd0:	cmp    $0x1a,%sil
   c1fd4:	setb   %sil
   c1fd8:	shl    $0x5,%sil
   c1fdc:	or     %dl,%sil
   c1fdf:	cmp    $0x72,%sil
   c1fe3:	mov    0x38(%rsp),%rsi
   c1fe8:	jne    c3189 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1cb9>
   c1fee:	movzbl 0x3(%rdi),%edx
   c1ff2:	lea    -0x41(%rdx),%r8d
   c1ff6:	cmp    $0x1a,%r8b
   c1ffa:	setb   %r8b
   c1ffe:	shl    $0x5,%r8b
   c2002:	or     %dl,%r8b
   c2005:	cmp    $0x73,%r8b
   c2009:	jne    c3189 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1cb9>
   c200f:	movzbl 0x4(%rdi),%edx
   c2013:	lea    -0x41(%rdx),%r8d
   c2017:	cmp    $0x1a,%r8b
   c201b:	setb   %r8b
   c201f:	shl    $0x5,%r8b
   c2023:	or     %dl,%r8b
   c2026:	cmp    $0x69,%r8b
   c202a:	jne    c3189 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1cb9>
   c2030:	movzbl 0x5(%rdi),%edx
   c2034:	lea    -0x41(%rdx),%r8d
   c2038:	cmp    $0x1a,%r8b
   c203c:	setb   %r8b
   c2040:	shl    $0x5,%r8b
   c2044:	or     %dl,%r8b
   c2047:	cmp    $0x6f,%r8b
   c204b:	jne    c3189 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1cb9>
   c2051:	movzbl 0x6(%rdi),%edx
   c2055:	lea    -0x41(%rdx),%r8d
   c2059:	cmp    $0x1a,%r8b
   c205d:	setb   %r8b
   c2061:	shl    $0x5,%r8b
   c2065:	or     %dl,%r8b
   c2068:	cmp    $0x6e,%r8b
   c206c:	jne    c3189 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1cb9>
   c2072:	mov    %r14,%rdi
   c2075:	mov    0x28(%rsp),%rdx
   c207a:	mov    %r13,%rcx
   c207d:	call   c37d0 <rvvdk_vmdk::descriptor::number>
   c2082:	mov    0x110(%rsp),%r8
   c208a:	mov    0x118(%rsp),%rax
   c2092:	cmp    $0x9,%r8
   c2096:	jne    c2fb7 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1ae7>
   c209c:	cmp    $0x1,%rax
   c20a0:	jne    c2fd1 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1b01>
   c20a6:	testb  $0x1,0xa0(%rsp)
   c20ae:	jne    c2e55 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1985>
   c20b4:	mov    $0x1,%al
   c20b6:	mov    %rax,0xa0(%rsp)
   c20be:	xor    %ebp,%ebp
   c20c0:	jmp    c18ac <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x3dc>
   c20c5:	movzbl (%rdi),%eax
   c20c8:	lea    -0x41(%rax),%ecx
   c20cb:	cmp    $0x1a,%cl
   c20ce:	setb   %dl
   c20d1:	shl    $0x5,%dl
   c20d4:	or     %al,%dl
   c20d6:	cmp    $0x70,%dl
   c20d9:	jne    c326e <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1d9e>
   c20df:	movzbl 0x1(%rdi),%edx
   c20e3:	lea    -0x41(%rdx),%esi
   c20e6:	cmp    $0x1a,%sil
   c20ea:	setb   %sil
   c20ee:	shl    $0x5,%sil
   c20f2:	or     %dl,%sil
   c20f5:	cmp    $0x61,%sil
   c20f9:	jne    c326e <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1d9e>
   c20ff:	movzbl 0x2(%rdi),%edx
   c2103:	lea    -0x41(%rdx),%esi
   c2106:	cmp    $0x1a,%sil
   c210a:	setb   %sil
   c210e:	shl    $0x5,%sil
   c2112:	or     %dl,%sil
   c2115:	cmp    $0x72,%sil
   c2119:	jne    c326e <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1d9e>
   c211f:	movzbl 0x3(%rdi),%edx
   c2123:	lea    -0x41(%rdx),%esi
   c2126:	cmp    $0x1a,%sil
   c212a:	setb   %sil
   c212e:	shl    $0x5,%sil
   c2132:	or     %dl,%sil
   c2135:	cmp    $0x65,%sil
   c2139:	jne    c326e <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1d9e>
   c213f:	movzbl 0x4(%rdi),%edx
   c2143:	lea    -0x41(%rdx),%esi
   c2146:	cmp    $0x1a,%sil
   c214a:	setb   %sil
   c214e:	shl    $0x5,%sil
   c2152:	or     %dl,%sil
   c2155:	cmp    $0x6e,%sil
   c2159:	jne    c326e <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1d9e>
   c215f:	movzbl 0x5(%rdi),%edx
   c2163:	lea    -0x41(%rdx),%esi
   c2166:	cmp    $0x1a,%sil
   c216a:	setb   %sil
   c216e:	shl    $0x5,%sil
   c2172:	or     %dl,%sil
   c2175:	cmp    $0x74,%sil
   c2179:	jne    c326e <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1d9e>
   c217f:	movzbl 0x6(%rdi),%edx
   c2183:	lea    -0x41(%rdx),%esi
   c2186:	cmp    $0x1a,%sil
   c218a:	setb   %sil
   c218e:	shl    $0x5,%sil
   c2192:	or     %dl,%sil
   c2195:	cmp    $0x63,%sil
   c2199:	jne    c326e <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1d9e>
   c219f:	movzbl 0x7(%rdi),%edx
   c21a3:	lea    -0x41(%rdx),%esi
   c21a6:	cmp    $0x1a,%sil
   c21aa:	setb   %sil
   c21ae:	shl    $0x5,%sil
   c21b2:	or     %dl,%sil
   c21b5:	cmp    $0x69,%sil
   c21b9:	jne    c326e <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1d9e>
   c21bf:	movzbl 0x8(%rdi),%edx
   c21c3:	lea    -0x41(%rdx),%esi
   c21c6:	cmp    $0x1a,%sil
   c21ca:	setb   %sil
   c21ce:	shl    $0x5,%sil
   c21d2:	or     %dl,%sil
   c21d5:	cmp    $0x64,%sil
   c21d9:	jne    c326e <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1d9e>
   c21df:	mov    %r14,%rdi
   c21e2:	mov    0x38(%rsp),%rsi
   c21e7:	mov    0x28(%rsp),%rdx
   c21ec:	mov    %r13,%rcx
   c21ef:	call   c3690 <rvvdk_vmdk::descriptor::cid>
   c21f4:	mov    0x110(%rsp),%r8
   c21fc:	cmp    $0x9,%r8
   c2200:	jne    c2f4f <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1a7f>
   c2206:	cmpl   $0xffffffff,0x118(%rsp)
   c220e:	jne    c3021 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1b51>
   c2214:	cmpl   $0x1,0x40(%rsp)
   c2219:	je     c2e55 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1985>
   c221f:	movl   $0x1,0x40(%rsp)
   c2227:	xor    %ebp,%ebp
   c2229:	jmp    c18ac <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x3dc>
   c222e:	cmpq   $0x6,0x28(%rsp)
   c2234:	je     c22c0 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0xdf0>
   c223a:	cmpq   $0x10,0x28(%rsp)
   c2240:	je     c228b <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0xdbb>
   c2242:	cmpq   $0x12,0x28(%rsp)
   c2248:	jne    c2e75 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x19a5>
   c224e:	mov    $0x12,%esi
   c2253:	mov    $0x12,%ecx
   c2258:	mov    %r15,%rdi
   c225b:	lea    -0xa14f6(%rip),%rdx        # 20d6c <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.12047894789963178251+0x30d>
   c2262:	call   c4050 <core::slice::ascii::<impl [u8]>::eq_ignore_ascii_case_chunks>
   c2267:	mov    $0x5,%r8d
   c226d:	mov    $0x1,%cl
   c226f:	test   %al,%al
   c2271:	jne    c2369 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0xe99>
   c2277:	cmpq   $0x6,0x28(%rsp)
   c227d:	je     c22c0 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0xdf0>
   c227f:	cmpq   $0x10,0x28(%rsp)
   c2285:	jne    c2e75 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x19a5>
   c228b:	mov    $0x10,%esi
   c2290:	mov    $0x10,%ecx
   c2295:	mov    %r15,%rdi
   c2298:	lea    -0xab59f(%rip),%rdx        # 16d00 <anon.e9ff7b7b9ea76053b3f0064765461615.59.llvm.11370392988746934037+0xb0>
   c229f:	call   c4050 <core::slice::ascii::<impl [u8]>::eq_ignore_ascii_case_chunks>
   c22a4:	mov    $0x5,%r8d
   c22aa:	mov    $0x1,%cl
   c22ac:	test   %al,%al
   c22ae:	jne    c2369 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0xe99>
   c22b4:	cmpq   $0x6,0x28(%rsp)
   c22ba:	jne    c2e75 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x19a5>
   c22c0:	movzbl (%r15),%eax
   c22c4:	lea    -0x41(%rax),%ecx
   c22c7:	cmp    $0x1a,%cl
   c22ca:	setb   %cl
   c22cd:	shl    $0x5,%cl
   c22d0:	or     %al,%cl
   c22d2:	cmp    $0x63,%cl
   c22d5:	jne    c2e75 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x19a5>
   c22db:	movzbl 0x1(%r15),%eax
   c22e0:	lea    -0x41(%rax),%ecx
   c22e3:	cmp    $0x1a,%cl
   c22e6:	setb   %cl
   c22e9:	shl    $0x5,%cl
   c22ec:	or     %al,%cl
   c22ee:	cmp    $0x75,%cl
   c22f1:	jne    c2e75 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x19a5>
   c22f7:	movzbl 0x2(%r15),%eax
   c22fc:	lea    -0x41(%rax),%ecx
   c22ff:	cmp    $0x1a,%cl
   c2302:	setb   %cl
   c2305:	shl    $0x5,%cl
   c2308:	or     %al,%cl
   c230a:	cmp    $0x73,%cl
   c230d:	jne    c2e75 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x19a5>
   c2313:	movzbl 0x3(%r15),%eax
   c2318:	lea    -0x41(%rax),%ecx
   c231b:	cmp    $0x1a,%cl
   c231e:	setb   %cl
   c2321:	shl    $0x5,%cl
   c2324:	or     %al,%cl
   c2326:	cmp    $0x74,%cl
   c2329:	jne    c2e75 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x19a5>
   c232f:	movzbl 0x4(%r15),%eax
   c2334:	lea    -0x41(%rax),%ecx
   c2337:	cmp    $0x1a,%cl
   c233a:	setb   %cl
   c233d:	shl    $0x5,%cl
   c2340:	or     %al,%cl
   c2342:	cmp    $0x6f,%cl
   c2345:	jne    c2e75 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x19a5>
   c234b:	movzbl 0x5(%r15),%eax
   c2350:	lea    -0x41(%rax),%ecx
   c2353:	cmp    $0x1a,%cl
   c2356:	setb   %dl
   c2359:	shl    $0x5,%dl
   c235c:	or     %al,%dl
   c235e:	mov    $0x2,%cl
   c2360:	cmp    $0x6d,%dl
   c2363:	jne    c2e75 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x19a5>
   c2369:	cmpb   $0x5,0x17(%rsp)
   c236e:	jne    c2e55 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1985>
   c2374:	xor    %ebp,%ebp
   c2376:	mov    %cl,0x17(%rsp)
   c237a:	jmp    c18ac <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x3dc>
   c237f:	cmp    $0x8,%r15
   c2383:	je     c24c1 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0xff1>
   c2389:	cmp    $0xf,%r15
   c238d:	jne    c2fa0 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1ad0>
   c2393:	movzbl 0x4(%rdi),%eax
   c2397:	lea    -0x41(%rax),%ecx
   c239a:	cmp    $0x1a,%cl
   c239d:	setb   %cl
   c23a0:	shl    $0x5,%cl
   c23a3:	or     %al,%cl
   c23a5:	cmp    $0x61,%cl
   c23a8:	jne    c2fa0 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1ad0>
   c23ae:	movzbl 0x5(%rdi),%eax
   c23b2:	lea    -0x41(%rax),%ecx
   c23b5:	cmp    $0x1a,%cl
   c23b8:	setb   %cl
   c23bb:	shl    $0x5,%cl
   c23be:	or     %al,%cl
   c23c0:	cmp    $0x64,%cl
   c23c3:	jne    c2fa0 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1ad0>
   c23c9:	movzbl 0x6(%rdi),%eax
   c23cd:	lea    -0x41(%rax),%ecx
   c23d0:	cmp    $0x1a,%cl
   c23d3:	setb   %cl
   c23d6:	shl    $0x5,%cl
   c23d9:	or     %al,%cl
   c23db:	cmp    $0x61,%cl
   c23de:	jne    c2fa0 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1ad0>
   c23e4:	movzbl 0x7(%rdi),%eax
   c23e8:	lea    -0x41(%rax),%ecx
   c23eb:	cmp    $0x1a,%cl
   c23ee:	setb   %cl
   c23f1:	shl    $0x5,%cl
   c23f4:	or     %al,%cl
   c23f6:	cmp    $0x70,%cl
   c23f9:	jne    c2fa0 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1ad0>
   c23ff:	movzbl 0x8(%rdi),%eax
   c2403:	lea    -0x41(%rax),%ecx
   c2406:	cmp    $0x1a,%cl
   c2409:	setb   %cl
   c240c:	shl    $0x5,%cl
   c240f:	or     %al,%cl
   c2411:	cmp    $0x74,%cl
   c2414:	jne    c2fa0 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1ad0>
   c241a:	movzbl 0x9(%rdi),%eax
   c241e:	lea    -0x41(%rax),%ecx
   c2421:	cmp    $0x1a,%cl
   c2424:	setb   %cl
   c2427:	shl    $0x5,%cl
   c242a:	or     %al,%cl
   c242c:	cmp    $0x65,%cl
   c242f:	jne    c2fa0 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1ad0>
   c2435:	movzbl 0xa(%rdi),%eax
   c2439:	lea    -0x41(%rax),%ecx
   c243c:	cmp    $0x1a,%cl
   c243f:	setb   %cl
   c2442:	shl    $0x5,%cl
   c2445:	or     %al,%cl
   c2447:	cmp    $0x72,%cl
   c244a:	jne    c2fa0 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1ad0>
   c2450:	movzbl 0xb(%rdi),%eax
   c2454:	lea    -0x41(%rax),%ecx
   c2457:	cmp    $0x1a,%cl
   c245a:	setb   %cl
   c245d:	shl    $0x5,%cl
   c2460:	or     %al,%cl
   c2462:	cmp    $0x74,%cl
   c2465:	jne    c2fa0 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1ad0>
   c246b:	movzbl 0xc(%rdi),%eax
   c246f:	lea    -0x41(%rax),%ecx
   c2472:	cmp    $0x1a,%cl
   c2475:	setb   %cl
   c2478:	shl    $0x5,%cl
   c247b:	or     %al,%cl
   c247d:	cmp    $0x79,%cl
   c2480:	jne    c2fa0 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1ad0>
   c2486:	movzbl 0xd(%rdi),%eax
   c248a:	lea    -0x41(%rax),%ecx
   c248d:	cmp    $0x1a,%cl
   c2490:	setb   %cl
   c2493:	shl    $0x5,%cl
   c2496:	or     %al,%cl
   c2498:	cmp    $0x70,%cl
   c249b:	jne    c2fa0 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1ad0>
   c24a1:	movzbl 0xe(%rdi),%eax
   c24a5:	lea    -0x41(%rax),%ecx
   c24a8:	cmp    $0x1a,%cl
   c24ab:	setb   %cl
   c24ae:	shl    $0x5,%cl
   c24b1:	or     %al,%cl
   c24b3:	cmp    $0x65,%cl
   c24b6:	je     c25a5 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x10d5>
   c24bc:	jmp    c2fa0 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1ad0>
   c24c1:	movzbl 0x4(%rdi),%eax
   c24c5:	lea    -0x41(%rax),%ecx
   c24c8:	cmp    $0x1a,%cl
   c24cb:	setb   %cl
   c24ce:	shl    $0x5,%cl
   c24d1:	or     %al,%cl
   c24d3:	cmp    $0x75,%cl
   c24d6:	jne    c2fa0 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1ad0>
   c24dc:	movzbl 0x5(%rdi),%eax
   c24e0:	lea    -0x41(%rax),%ecx
   c24e3:	cmp    $0x1a,%cl
   c24e6:	setb   %cl
   c24e9:	shl    $0x5,%cl
   c24ec:	or     %al,%cl
   c24ee:	cmp    $0x75,%cl
   c24f1:	jne    c2fa0 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1ad0>
   c24f7:	movzbl 0x6(%rdi),%eax
   c24fb:	lea    -0x41(%rax),%ecx
   c24fe:	cmp    $0x1a,%cl
   c2501:	setb   %cl
   c2504:	shl    $0x5,%cl
   c2507:	or     %al,%cl
   c2509:	cmp    $0x69,%cl
   c250c:	jne    c2fa0 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1ad0>
   c2512:	movzbl 0x7(%rdi),%eax
   c2516:	lea    -0x41(%rax),%ecx
   c2519:	cmp    $0x1a,%cl
   c251c:	setb   %cl
   c251f:	shl    $0x5,%cl
   c2522:	or     %al,%cl
   c2524:	cmp    $0x64,%cl
   c2527:	je     c25a5 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x10d5>
   c2529:	jmp    c2fa0 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1ad0>
   c252e:	mov    $0x11,%esi
   c2533:	mov    $0x11,%ecx
   c2538:	lea    -0xa1933(%rip),%rdx        # 20c0c <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.12047894789963178251+0x1ad>
   c253f:	jmp    c2598 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x10c8>
   c2541:	mov    $0x14,%esi
   c2546:	mov    $0x14,%ecx
   c254b:	mov    %rdi,%r14
   c254e:	lea    -0xa1971(%rip),%rdx        # 20be4 <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.12047894789963178251+0x185>
   c2555:	call   c4050 <core::slice::ascii::<impl [u8]>::eq_ignore_ascii_case_chunks>
   c255a:	test   %al,%al
   c255c:	jne    c25a5 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x10d5>
   c255e:	mov    $0x14,%esi
   c2563:	mov    $0x14,%ecx
   c2568:	mov    %r14,%rdi
   c256b:	lea    -0xa197a(%rip),%rdx        # 20bf8 <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.12047894789963178251+0x199>
   c2572:	jmp    c2598 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x10c8>
   c2574:	mov    $0x12,%esi
   c2579:	mov    $0x12,%ecx
   c257e:	lea    -0xa19b3(%rip),%rdx        # 20bd2 <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.12047894789963178251+0x173>
   c2585:	jmp    c2598 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x10c8>
   c2587:	mov    $0x16,%esi
   c258c:	mov    $0x16,%ecx
   c2591:	lea    -0xa19dc(%rip),%rdx        # 20bbc <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.12047894789963178251+0x15d>
   c2598:	call   c4050 <core::slice::ascii::<impl [u8]>::eq_ignore_ascii_case_chunks>
   c259d:	test   %al,%al
   c259f:	je     c2fa0 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1ad0>
   c25a5:	cmpq   $0x1,0x48(%rsp)
   c25ab:	jne    c34be <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1fee>
   c25b1:	mov    0x58(%rsp),%rbp
   c25b6:	mov    0x60(%rsp),%rax
   c25bb:	mov    %rax,%r12
   c25be:	shl    $0x5,%r12
   c25c2:	mov    %rax,0x48(%rsp)
   c25c7:	test   %rax,%rax
   c25ca:	je     c2670 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x11a0>
   c25d0:	cmp    $0xf,%r15
   c25d4:	jbe    c260d <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x113d>
   c25d6:	xor    %r14d,%r14d
   c25d9:	jmp    c25e8 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1118>
   c25db:	add    $0x20,%r14
   c25df:	cmp    %r14,%r12
   c25e2:	je     c2670 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x11a0>
   c25e8:	cmp    %r15,0x8(%rbp,%r14,1)
   c25ed:	jne    c25db <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x110b>
   c25ef:	mov    0x0(%rbp,%r14,1),%rdi
   c25f4:	mov    %r15,%rsi
   c25f7:	mov    0x18(%rsp),%rdx
   c25fc:	mov    %r15,%rcx
   c25ff:	call   c4050 <core::slice::ascii::<impl [u8]>::eq_ignore_ascii_case_chunks>
   c2604:	test   %al,%al
   c2606:	je     c25db <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x110b>
   c2608:	jmp    c2e55 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1985>
   c260d:	lea    (%r12,%rbp,1),%rax
   c2611:	mov    %rbp,%rcx
   c2614:	jmp    c261f <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x114f>
   c2616:	add    $0x20,%rcx
   c261a:	cmp    %rax,%rcx
   c261d:	je     c2670 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x11a0>
   c261f:	cmp    %r15,0x8(%rcx)
   c2623:	jne    c2616 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1146>
   c2625:	mov    (%rcx),%rdx
   c2628:	xor    %esi,%esi
   c262a:	cmp    %rsi,%r15
   c262d:	je     c2e55 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1985>
   c2633:	movzbl (%rdx,%rsi,1),%edi
   c2637:	lea    -0x41(%rdi),%r8d
   c263b:	cmp    $0x1a,%r8b
   c263f:	setb   %r8b
   c2643:	shl    $0x5,%r8b
   c2647:	or     %dil,%r8b
   c264a:	mov    0x18(%rsp),%rdi
   c264f:	movzbl (%rdi,%rsi,1),%edi
   c2653:	lea    -0x41(%rdi),%r9d
   c2657:	cmp    $0x1a,%r9b
   c265b:	setb   %r9b
   c265f:	shl    $0x5,%r9b
   c2663:	or     %dil,%r9b
   c2666:	inc    %rsi
   c2669:	cmp    %r9b,%r8b
   c266c:	je     c262a <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x115a>
   c266e:	jmp    c2616 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1146>
   c2670:	mov    0x48(%rsp),%rax
   c2675:	cmp    0x1b0(%rsp),%rax
   c267d:	jae    c34db <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x200b>
   c2683:	cmp    0x50(%rsp),%rax
   c2688:	lea    0x110(%rsp),%r14
   c2690:	jne    c26a2 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x11d2>
   c2692:	lea    0x50(%rsp),%rdi
   c2697:	call   *0x1f65ab(%rip)        # 2b8c48 <_DYNAMIC+0x900>
   c269d:	mov    0x58(%rsp),%rbp
   c26a2:	mov    0x18(%rsp),%rax
   c26a7:	mov    %rax,0x0(%rbp,%r12,1)
   c26ac:	mov    %r15,0x8(%rbp,%r12,1)
   c26b1:	mov    0x38(%rsp),%rax
   c26b6:	mov    %rax,0x10(%rbp,%r12,1)
   c26bb:	mov    0x28(%rsp),%rax
   c26c0:	mov    %rax,0x18(%rbp,%r12,1)
   c26c5:	mov    0x48(%rsp),%rax
   c26ca:	inc    %rax
   c26cd:	mov    %rax,0x60(%rsp)
   c26d2:	mov    $0x2,%ebp
   c26d7:	jmp    c18ac <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x3dc>
   c26dc:	cmp    $0x2,%ebp
   c26df:	je     c305e <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1b8e>
   c26e5:	mov    0x88(%rsp),%r12
   c26ed:	cmp    0x1b8(%rsp),%r12
   c26f5:	jae    c3070 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1ba0>
   c26fb:	mov    %r12,0xb0(%rsp)
   c2703:	mov    $0xd,%r12d
   c2709:	test   %rcx,%rcx
   c270c:	jne    c308a <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1bba>
   c2712:	cmp    $0x3,%rsi
   c2716:	mov    0x18(%rsp),%rbp
   c271b:	jb     c308a <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1bba>
   c2721:	test   %rdx,%rdx
   c2724:	jne    c308a <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1bba>
   c272a:	cmpq   $0x0,0x48(%rsp)
   c2730:	jne    c308a <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1bba>
   c2736:	cmp    $0x6,%r15
   c273a:	je     c2783 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x12b3>
   c273c:	cmp    $0x2,%r15
   c2740:	jne    c2f46 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1a76>
   c2746:	movzbl 0x0(%rbp),%ecx
   c274a:	lea    -0x41(%rcx),%edx
   c274d:	cmp    $0x1a,%dl
   c2750:	setb   %dl
   c2753:	shl    $0x5,%dl
   c2756:	or     %cl,%dl
   c2758:	cmp    $0x72,%dl
   c275b:	jne    c2f46 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1a76>
   c2761:	movzbl 0x1(%rbp),%ecx
   c2765:	lea    -0x41(%rcx),%edx
   c2768:	cmp    $0x1a,%dl
   c276b:	setb   %dl
   c276e:	shl    $0x5,%dl
   c2771:	or     %cl,%dl
   c2773:	cmp    $0x77,%dl
   c2776:	jne    c2f46 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1a76>
   c277c:	xor    %ebp,%ebp
   c277e:	jmp    c2828 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1358>
   c2783:	movzbl 0x0(%rbp),%ecx
   c2787:	lea    -0x41(%rcx),%edx
   c278a:	cmp    $0x1a,%dl
   c278d:	setb   %dl
   c2790:	shl    $0x5,%dl
   c2793:	or     %cl,%dl
   c2795:	cmp    $0x72,%dl
   c2798:	jne    c2f46 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1a76>
   c279e:	movzbl 0x1(%rbp),%ecx
   c27a2:	lea    -0x41(%rcx),%edx
   c27a5:	cmp    $0x1a,%dl
   c27a8:	setb   %dl
   c27ab:	shl    $0x5,%dl
   c27ae:	or     %cl,%dl
   c27b0:	cmp    $0x64,%dl
   c27b3:	jne    c2f46 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1a76>
   c27b9:	movzbl 0x2(%rbp),%ecx
   c27bd:	lea    -0x41(%rcx),%edx
   c27c0:	cmp    $0x1a,%dl
   c27c3:	setb   %dl
   c27c6:	shl    $0x5,%dl
   c27c9:	or     %cl,%dl
   c27cb:	cmp    $0x6f,%dl
   c27ce:	jne    c2f46 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1a76>
   c27d4:	movzbl 0x3(%rbp),%ecx
   c27d8:	lea    -0x41(%rcx),%edx
   c27db:	cmp    $0x1a,%dl
   c27de:	setb   %dl
   c27e1:	shl    $0x5,%dl
   c27e4:	or     %cl,%dl
   c27e6:	cmp    $0x6e,%dl
   c27e9:	jne    c2f46 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1a76>
   c27ef:	movzbl 0x4(%rbp),%ecx
   c27f3:	lea    -0x41(%rcx),%edx
   c27f6:	cmp    $0x1a,%dl
   c27f9:	setb   %dl
   c27fc:	shl    $0x5,%dl
   c27ff:	or     %cl,%dl
   c2801:	cmp    $0x6c,%dl
   c2804:	jne    c2f46 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1a76>
   c280a:	movzbl 0x5(%rbp),%ecx
   c280e:	lea    -0x41(%rcx),%edx
   c2811:	cmp    $0x1a,%dl
   c2814:	setb   %dl
   c2817:	shl    $0x5,%dl
   c281a:	or     %cl,%dl
   c281c:	mov    $0x1,%bpl
   c281f:	cmp    $0x79,%dl
   c2822:	jne    c2f46 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1a76>
   c2828:	test   %rax,%rax
   c282b:	je     c2dde <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x190e>
   c2831:	add    $0xfffffffffffffffd,%rsi
   c2835:	xor    %ecx,%ecx
   c2837:	cmp    %rcx,%rax
   c283a:	je     c2851 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1381>
   c283c:	movzbl (%r14,%rcx,1),%edx
   c2841:	add    $0xc6,%dl
   c2844:	inc    %rcx
   c2847:	cmp    $0xf6,%dl
   c284a:	jae    c2837 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1367>
   c284c:	jmp    c2dde <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x190e>
   c2851:	movzbl (%r14),%ecx
   c2855:	cmp    $0x1,%rax
   c2859:	jne    c286d <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x139d>
   c285b:	cmp    $0x2b,%ecx
   c285e:	je     c2dde <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x190e>
   c2864:	cmp    $0x2d,%ecx
   c2867:	je     c2dde <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x190e>
   c286d:	xor    %r15d,%r15d
   c2870:	cmp    $0x2b,%ecx
   c2873:	sete   %r15b
   c2877:	mov    %rax,%rcx
   c287a:	sub    %r15,%rcx
   c287d:	add    %r15,%r14
   c2880:	neg    %r15
   c2883:	cmp    $0x11,%rcx
   c2887:	jae    c2a01 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1531>
   c288d:	test   %rcx,%rcx
   c2890:	je     c34ed <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x201d>
   c2896:	add    %rax,%r15
   c2899:	neg    %r15
   c289c:	xor    %ecx,%ecx
   c289e:	xor    %eax,%eax
   c28a0:	movzbl (%r14,%rcx,1),%edx
   c28a5:	add    $0xffffffd0,%edx
   c28a8:	cmp    $0x9,%edx
   c28ab:	ja     c2dde <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x190e>
   c28b1:	lea    (%rax,%rax,4),%rax
   c28b5:	mov    %edx,%edx
   c28b7:	lea    (%rdx,%rax,2),%rax
   c28bb:	inc    %rcx
   c28be:	mov    %r15,%rdx
   c28c1:	add    %rcx,%rdx
   c28c4:	jne    c28a0 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x13d0>
   c28c6:	mov    %rax,%rcx
   c28c9:	shr    $0x37,%rcx
   c28cd:	jne    c3595 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x20c5>
   c28d3:	test   %rax,%rax
   c28d6:	mov    0x38(%rsp),%r15
   c28db:	je     c3523 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x2053>
   c28e1:	mov    $0xb,%r12d
   c28e7:	mov    $0x5,%edx
   c28ec:	cmpq   $0x4,0x28(%rsp)
   c28f2:	jne    c34aa <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1fda>
   c28f8:	mov    %rax,%rcx
   c28fb:	shl    $0x9,%rcx
   c28ff:	mov    %rcx,0x20(%rsp)
   c2904:	movzbl (%r15),%r14d
   c2908:	lea    -0x41(%r14),%ecx
   c290c:	cmp    $0x1a,%cl
   c290f:	setb   %cl
   c2912:	shl    $0x5,%cl
   c2915:	or     %r14b,%cl
   c2918:	cmp    $0x7a,%cl
   c291b:	je     c2a4a <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x157a>
   c2921:	movzbl %cl,%ecx
   c2924:	cmp    $0x66,%ecx
   c2927:	jne    c34aa <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1fda>
   c292d:	movzbl 0x1(%r15),%ecx
   c2932:	lea    -0x41(%rcx),%r14d
   c2936:	cmp    $0x1a,%r14b
   c293a:	setb   %r14b
   c293e:	shl    $0x5,%r14b
   c2942:	or     %cl,%r14b
   c2945:	cmp    $0x6c,%r14b
   c2949:	jne    c34aa <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1fda>
   c294f:	movzbl 0x2(%r15),%ecx
   c2954:	lea    -0x41(%rcx),%r14d
   c2958:	cmp    $0x1a,%r14b
   c295c:	setb   %r14b
   c2960:	shl    $0x5,%r14b
   c2964:	or     %cl,%r14b
   c2967:	cmp    $0x61,%r14b
   c296b:	jne    c34aa <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1fda>
   c2971:	movzbl 0x3(%r15),%ecx
   c2976:	lea    -0x41(%rcx),%r14d
   c297a:	cmp    $0x1a,%r14b
   c297e:	setb   %r14b
   c2982:	shl    $0x5,%r14b
   c2986:	or     %cl,%r14b
   c2989:	cmp    $0x74,%r14b
   c298d:	jne    c34aa <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1fda>
   c2993:	mov    $0x21,%r12d
   c2999:	cmp    $0x2,%rsi
   c299d:	jne    c3549 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x2079>
   c29a3:	cmp    $0x1,%r10
   c29a7:	jne    c3549 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x2079>
   c29ad:	test   %r11,%r11
   c29b0:	jne    c3549 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x2079>
   c29b6:	mov    $0xe,%r12d
   c29bc:	mov    0x68(%rsp),%rcx
   c29c1:	test   %rcx,%rcx
   c29c4:	je     c3555 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x2085>
   c29ca:	cmp    0x1a8(%rsp),%rcx
   c29d2:	ja     c3589 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x20b9>
   c29d8:	test   %r9,%r9
   c29db:	je     c3054 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1b84>
   c29e1:	xor    %ecx,%ecx
   c29e3:	cmp    %rcx,%r9
   c29e6:	je     c2b58 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1688>
   c29ec:	movzbl (%r8,%rcx,1),%edx
   c29f1:	add    $0xc6,%dl
   c29f4:	inc    %rcx
   c29f7:	cmp    $0xf6,%dl
   c29fa:	jae    c29e3 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1513>
   c29fc:	jmp    c3054 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1b84>
   c2a01:	add    %rax,%r15
   c2a04:	neg    %r15
   c2a07:	xor    %ecx,%ecx
   c2a09:	xor    %eax,%eax
   c2a0b:	mov    %r15,%rdx
   c2a0e:	add    %rcx,%rdx
   c2a11:	je     c28c6 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x13f6>
   c2a17:	mov    $0xa,%edx
   c2a1c:	mul    %rdx
   c2a1f:	jo     c2dde <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x190e>
   c2a25:	movzbl (%r14,%rcx,1),%r12d
   c2a2a:	add    $0xffffffd0,%r12d
   c2a2e:	add    %r12,%rax
   c2a31:	setb   %dl
   c2a34:	cmp    $0x9,%r12d
   c2a38:	ja     c2dde <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x190e>
   c2a3e:	inc    %rcx
   c2a41:	test   %dl,%dl
   c2a43:	je     c2a0b <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x153b>
   c2a45:	jmp    c2dde <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x190e>
   c2a4a:	movzbl 0x1(%r15),%eax
   c2a4f:	lea    -0x41(%rax),%ecx
   c2a52:	cmp    $0x1a,%cl
   c2a55:	setb   %cl
   c2a58:	shl    $0x5,%cl
   c2a5b:	or     %al,%cl
   c2a5d:	cmp    $0x65,%cl
   c2a60:	jne    c34aa <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1fda>
   c2a66:	movzbl 0x2(%r15),%eax
   c2a6b:	lea    -0x41(%rax),%ecx
   c2a6e:	cmp    $0x1a,%cl
   c2a71:	setb   %cl
   c2a74:	shl    $0x5,%cl
   c2a77:	or     %al,%cl
   c2a79:	cmp    $0x72,%cl
   c2a7c:	mov    $0x7,%r8d
   c2a82:	jne    c34aa <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1fda>
   c2a88:	movzbl 0x3(%r15),%eax
   c2a8d:	lea    -0x41(%rax),%ecx
   c2a90:	cmp    $0x1a,%cl
   c2a93:	setb   %cl
   c2a96:	shl    $0x5,%cl
   c2a99:	or     %al,%cl
   c2a9b:	cmp    $0x6f,%cl
   c2a9e:	jne    c34aa <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1fda>
   c2aa4:	test   %rsi,%rsi
   c2aa7:	mov    0x20(%rsp),%r14
   c2aac:	jne    c3532 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x2062>
   c2ab2:	mov    $0x1,%r15d
   c2ab8:	cmpb   $0x2,0x17(%rsp)
   c2abd:	jne    c350a <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x203a>
   c2ac3:	add    0x70(%rsp),%r14
   c2ac8:	jb     c2cc0 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x17f0>
   c2ace:	mov    0xb0(%rsp),%r12
   c2ad6:	cmp    0x78(%rsp),%r12
   c2adb:	jne    c2ae8 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1618>
   c2add:	lea    0x78(%rsp),%rdi
   c2ae2:	call   *0x1f6168(%rip)        # 2b8c50 <_DYNAMIC+0x908>
   c2ae8:	mov    0x80(%rsp),%rax
   c2af0:	imul   $0x38,%r12,%rcx
   c2af4:	mov    %r15,(%rax,%rcx,1)
   c2af8:	mov    0x30(%rsp),%rdx
   c2afd:	mov    %rdx,0x8(%rax,%rcx,1)
   c2b02:	mov    0x98(%rsp),%rdx
   c2b0a:	mov    %rdx,0x10(%rax,%rcx,1)
   c2b0f:	mov    0x90(%rsp),%rdx
   c2b17:	mov    %rdx,0x18(%rax,%rcx,1)
   c2b1c:	mov    0x70(%rsp),%rdx
   c2b21:	mov    %rdx,0x20(%rax,%rcx,1)
   c2b26:	mov    0x20(%rsp),%rdx
   c2b2b:	mov    %rdx,0x28(%rax,%rcx,1)
   c2b30:	mov    %bpl,0x30(%rax,%rcx,1)
   c2b35:	inc    %r12
   c2b38:	mov    %r12,0x88(%rsp)
   c2b40:	mov    $0x1,%ebp
   c2b45:	cmpb   $0x0,0x109(%rsp)
   c2b4d:	je     c1647 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x177>
   c2b53:	jmp    c2d41 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1871>
   c2b58:	mov    %rax,%rsi
   c2b5b:	movzbl (%r8),%eax
   c2b5f:	cmp    $0x1,%r9
   c2b63:	jne    c2b77 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x16a7>
   c2b65:	cmp    $0x2b,%eax
   c2b68:	je     c3054 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1b84>
   c2b6e:	cmp    $0x2d,%eax
   c2b71:	je     c3054 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1b84>
   c2b77:	xor    %r10d,%r10d
   c2b7a:	cmp    $0x2b,%eax
   c2b7d:	sete   %r10b
   c2b81:	mov    %r9,%rax
   c2b84:	sub    %r10,%rax
   c2b87:	add    %r10,%r8
   c2b8a:	neg    %r10
   c2b8d:	cmp    $0x11,%rax
   c2b91:	jae    c2be0 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1710>
   c2b93:	test   %rax,%rax
   c2b96:	je     c2c25 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1755>
   c2b9c:	add    %r9,%r10
   c2b9f:	neg    %r10
   c2ba2:	xor    %ecx,%ecx
   c2ba4:	xor    %eax,%eax
   c2ba6:	movzbl (%r8,%rcx,1),%edx
   c2bab:	add    $0xffffffd0,%edx
   c2bae:	cmp    $0x9,%edx
   c2bb1:	ja     c3054 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1b84>
   c2bb7:	lea    (%rax,%rax,4),%rax
   c2bbb:	mov    %edx,%edx
   c2bbd:	lea    (%rdx,%rax,2),%rax
   c2bc1:	inc    %rcx
   c2bc4:	mov    %r10,%rdx
   c2bc7:	add    %rcx,%rdx
   c2bca:	jne    c2ba6 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x16d6>
   c2bcc:	movabs $0x7fffffffffffff,%rcx
   c2bd6:	cmp    %rcx,%rax
   c2bd9:	jbe    c2c27 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1757>
   c2bdb:	jmp    c35c8 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x20f8>
   c2be0:	add    %r9,%r10
   c2be3:	neg    %r10
   c2be6:	xor    %ecx,%ecx
   c2be8:	xor    %eax,%eax
   c2bea:	mov    %r10,%rdx
   c2bed:	add    %rcx,%rdx
   c2bf0:	je     c2bcc <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x16fc>
   c2bf2:	mov    $0xa,%edx
   c2bf7:	mul    %rdx
   c2bfa:	jo     c3054 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1b84>
   c2c00:	movzbl (%r8,%rcx,1),%r9d
   c2c05:	add    $0xffffffd0,%r9d
   c2c09:	add    %r9,%rax
   c2c0c:	setb   %dl
   c2c0f:	cmp    $0x9,%r9d
   c2c13:	ja     c3054 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1b84>
   c2c19:	inc    %rcx
   c2c1c:	test   %dl,%dl
   c2c1e:	je     c2bea <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x171a>
   c2c20:	jmp    c3054 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1b84>
   c2c25:	xor    %eax,%eax
   c2c27:	mov    %rax,%rcx
   c2c2a:	shl    $0x9,%rcx
   c2c2e:	mov    %rcx,0x30(%rsp)
   c2c33:	add    0x20(%rsp),%rcx
   c2c38:	jb     c35cf <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x20ff>
   c2c3e:	cmpb   $0x2,0x17(%rsp)
   c2c43:	jae    c2c63 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1793>
   c2c45:	test   %rax,%rax
   c2c48:	jne    c3514 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x2044>
   c2c4e:	cmpb   $0x0,0x17(%rsp)
   c2c53:	je     c2c72 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x17a2>
   c2c55:	cmp    $0x400000,%rsi
   c2c5c:	jbe    c2c81 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x17b1>
   c2c5e:	jmp    c35aa <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x20da>
   c2c63:	movzbl 0x17(%rsp),%eax
   c2c68:	cmp    $0x2,%eax
   c2c6b:	je     c2c8a <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x17ba>
   c2c6d:	jmp    c356c <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x209c>
   c2c72:	cmpq   $0x0,0xb0(%rsp)
   c2c7b:	jne    c35b9 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x20e9>
   c2c81:	movq   $0x0,0x30(%rsp)
   c2c8a:	xor    %r15d,%r15d
   c2c8d:	mov    %rdi,0x98(%rsp)
   c2c95:	mov    0x68(%rsp),%rax
   c2c9a:	mov    %rax,0x90(%rsp)
   c2ca2:	mov    $0x7,%r8d
   c2ca8:	mov    0x20(%rsp),%r14
   c2cad:	add    0x70(%rsp),%r14
   c2cb2:	jae    c2ace <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x15fe>
   c2cb8:	jmp    c2cc0 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x17f0>
   c2cba:	mov    $0x1,%r8d
   c2cc0:	movabs $0x8000000000000000,%r15
   c2cca:	mov    0x8(%rsp),%rbp
   c2ccf:	mov    %r8,%r14
   c2cd2:	mov    0x50(%rsp),%rsi
   c2cd7:	test   %rsi,%rsi
   c2cda:	je     c2cf0 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1820>
   c2cdc:	mov    0x58(%rsp),%rdi
   c2ce1:	shl    $0x5,%rsi
   c2ce5:	mov    $0x8,%edx
   c2cea:	call   *0x1f58a8(%rip)        # 2b8598 <_DYNAMIC+0x250>
   c2cf0:	mov    0x78(%rsp),%rax
   c2cf5:	test   %rax,%rax
   c2cf8:	je     c2d11 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1841>
   c2cfa:	mov    0x80(%rsp),%rdi
   c2d02:	imul   $0x38,%rax,%rsi
   c2d06:	mov    $0x8,%edx
   c2d0b:	call   *0x1f5887(%rip)        # 2b8598 <_DYNAMIC+0x250>
   c2d11:	mov    %r14,%rcx
   c2d14:	mov    %rcx,0x8(%rbx)
   c2d18:	mov    %rbp,0x10(%rbx)
   c2d1c:	mov    %r12,0x18(%rbx)
   c2d20:	mov    %r13,0x20(%rbx)
   c2d24:	mov    %r15,(%rbx)
   c2d27:	mov    %rbx,%rax
   c2d2a:	add    $0x1c8,%rsp
   c2d31:	pop    %rbx
   c2d32:	pop    %r12
   c2d34:	pop    %r13
   c2d36:	pop    %r14
   c2d38:	pop    %r15
   c2d3a:	pop    %rbp
   c2d3b:	ret
   c2d3c:	mov    0x70(%rsp),%r14
   c2d41:	mov    $0x4,%r8d
   c2d47:	mov    $0x7,%r12d
   c2d4d:	cmpb   $0x1,0xa0(%rsp)
   c2d55:	jne    c2d9b <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x18cb>
   c2d57:	testb  $0x1,0x40(%rsp)
   c2d5c:	movabs $0x8000000000000000,%r15
   c2d66:	je     c2dc9 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x18f9>
   c2d68:	cmpl   $0x1,0x44(%rsp)
   c2d6d:	jne    c2e60 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1990>
   c2d73:	cmpb   $0x5,0x17(%rsp)
   c2d78:	jne    c2f14 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1a44>
   c2d7e:	lea    -0xa21ec(%rip),%rbp        # 20b99 <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.12047894789963178251+0x13a>
   c2d85:	mov    $0xa,%r12d
   c2d8b:	xor    %r13d,%r13d
   c2d8e:	jmp    c2ccf <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x17ff>
   c2d93:	xor    %r8d,%r8d
   c2d96:	jmp    c2cc0 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x17f0>
   c2d9b:	lea    -0xa222e(%rip),%rbp        # 20b74 <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.12047894789963178251+0x115>
   c2da2:	xor    %r13d,%r13d
   c2da5:	movabs $0x8000000000000000,%r15
   c2daf:	jmp    c2ccf <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x17ff>
   c2db4:	mov    %r15,0x8(%rsp)
   c2db9:	mov    %r14,%r13
   c2dbc:	mov    %rdx,%r12
   c2dbf:	mov    0x18(%rsp),%r8
   c2dc4:	jmp    c2cc0 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x17f0>
   c2dc9:	lea    -0xa2255(%rip),%rbp        # 20b7b <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.12047894789963178251+0x11c>
   c2dd0:	mov    $0x9,%r12d
   c2dd6:	xor    %r13d,%r13d
   c2dd9:	jmp    c2ccf <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x17ff>
   c2dde:	mov    0x20(%rsp),%rax
   c2de3:	mov    %rax,0x8(%rsp)
   c2de8:	mov    $0x6,%r8d
   c2dee:	jmp    c2cc0 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x17f0>
   c2df3:	lea    -0xa2177(%rip),%rax        # 20c83 <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.12047894789963178251+0x224>
   c2dfa:	mov    %rax,0x8(%rsp)
   c2dff:	mov    $0x14,%r12d
   c2e05:	jmp    c2cc0 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x17f0>
   c2e0a:	cmp    $0x12,%r15
   c2e0e:	jne    c2e86 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x19b6>
   c2e10:	lea    -0xa2293(%rip),%rdx        # 20b84 <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.12047894789963178251+0x125>
   c2e17:	mov    $0x12,%esi
   c2e1c:	mov    $0x12,%ecx
   c2e21:	call   c4050 <core::slice::ascii::<impl [u8]>::eq_ignore_ascii_case_chunks>
   c2e26:	lea    -0xa21d4(%rip),%rcx        # 20c59 <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.12047894789963178251+0x1fa>
   c2e2d:	lea    -0xa21cf(%rip),%rdx        # 20c65 <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.12047894789963178251+0x206>
   c2e34:	test   %al,%al
   c2e36:	cmovne %rcx,%rdx
   c2e3a:	mov    %rdx,0x8(%rsp)
   c2e3f:	movzbl %al,%eax
   c2e42:	lea    0xa(,%rax,2),%r12
   c2e4a:	mov    $0x5,%r8d
   c2e50:	jmp    c2cc0 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x17f0>
   c2e55:	mov    $0x3,%r8d
   c2e5b:	jmp    c2cc0 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x17f0>
   c2e60:	lea    -0xa22d1(%rip),%rbp        # 20b96 <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.12047894789963178251+0x137>
   c2e67:	mov    $0x3,%r12d
   c2e6d:	xor    %r13d,%r13d
   c2e70:	jmp    c2ccf <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x17ff>
   c2e75:	lea    -0xa222e(%rip),%rax        # 20c4e <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.12047894789963178251+0x1ef>
   c2e7c:	mov    %rax,0x8(%rsp)
   c2e81:	jmp    c2cc0 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x17f0>
   c2e86:	lea    -0xa2228(%rip),%rdx        # 20c65 <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.12047894789963178251+0x206>
   c2e8d:	cmp    $0xf,%r15
   c2e91:	ja     c2f04 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1a34>
   c2e93:	add    $0xfffffffffffffffd,%r15
   c2e97:	cmp    $0x7,%r15
   c2e9b:	ja     c2f04 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1a34>
   c2e9d:	lea    -0xa236c(%rip),%rax        # 20b38 <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.12047894789963178251+0xd9>
   c2ea4:	movslq (%rax,%r15,4),%rcx
   c2ea8:	add    %rax,%rcx
   c2eab:	jmp    *%rcx
   c2ead:	movzbl (%rdi),%eax
   c2eb0:	lea    -0x41(%rax),%ecx
   c2eb3:	cmp    $0x1a,%cl
   c2eb6:	setb   %cl
   c2eb9:	shl    $0x5,%cl
   c2ebc:	or     %al,%cl
   c2ebe:	lea    -0xa2260(%rip),%rdx        # 20c65 <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.12047894789963178251+0x206>
   c2ec5:	cmp    $0x63,%cl
   c2ec8:	jne    c2f04 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1a34>
   c2eca:	movzbl 0x1(%rdi),%eax
   c2ece:	lea    -0x41(%rax),%ecx
   c2ed1:	cmp    $0x1a,%cl
   c2ed4:	setb   %cl
   c2ed7:	shl    $0x5,%cl
   c2eda:	or     %al,%cl
   c2edc:	cmp    $0x69,%cl
   c2edf:	jne    c2f04 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1a34>
   c2ee1:	movzbl 0x2(%rdi),%eax
   c2ee5:	lea    -0x41(%rax),%ecx
   c2ee8:	cmp    $0x1a,%cl
   c2eeb:	setb   %cl
   c2eee:	shl    $0x5,%cl
   c2ef1:	or     %al,%cl
   c2ef3:	cmp    $0x64,%cl
   c2ef6:	jne    c2f04 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1a34>
   c2ef8:	lea    -0xa2290(%rip),%rax        # 20c6f <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.12047894789963178251+0x210>
   c2eff:	jmp    c2dfa <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x192a>
   c2f04:	mov    %rdx,0x8(%rsp)
   c2f09:	mov    $0x5,%r8d
   c2f0f:	jmp    c2cc0 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x17f0>
   c2f14:	mov    0x88(%rsp),%rbp
   c2f1c:	test   %rbp,%rbp
   c2f1f:	je     c2f91 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1ac1>
   c2f21:	mov    0x78(%rsp),%rax
   c2f26:	mov    0x80(%rsp),%rcx
   c2f2e:	mov    0x50(%rsp),%r12
   c2f33:	cmp    %r15,%rax
   c2f36:	jne    c2fee <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1b1e>
   c2f3c:	mov    0x58(%rsp),%r13
   c2f41:	jmp    c2d14 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1844>
   c2f46:	lea    -0xa2205(%rip),%rax        # 20d48 <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.12047894789963178251+0x2e9>
   c2f4d:	jmp    c2fa7 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1ad7>
   c2f4f:	mov    0x118(%rsp),%rax
   c2f57:	mov    %rax,0x8(%rsp)
   c2f5c:	mov    0x120(%rsp),%r12
   c2f64:	mov    0x128(%rsp),%r13
   c2f6c:	jmp    c2cc0 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x17f0>
   c2f71:	xor    %ecx,%ecx
   c2f73:	cmp    $0x38,%al
   c2f75:	setne  %cl
   c2f78:	lea    0x3(,%rcx,2),%r8
   c2f80:	lea    -0xab9ef(%rip),%rax        # 17598 <anon.786f9ca1aa6bf68944dde0bec4ae4f7b.2.llvm.4745231054059156856+0x2d8>
   c2f87:	mov    %rax,0x8(%rsp)
   c2f8c:	jmp    c2cc0 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x17f0>
   c2f91:	lea    -0xa23f5(%rip),%rbp        # 20ba3 <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.12047894789963178251+0x144>
   c2f98:	xor    %r13d,%r13d
   c2f9b:	jmp    c2ccf <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x17ff>
   c2fa0:	lea    -0xa238a(%rip),%rax        # 20c1d <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.12047894789963178251+0x1be>
   c2fa7:	mov    %rax,0x8(%rsp)
   c2fac:	mov    $0x5,%r8d
   c2fb2:	jmp    c2cc0 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x17f0>
   c2fb7:	mov    0x120(%rsp),%r12
   c2fbf:	mov    0x128(%rsp),%r13
   c2fc7:	mov    %rax,0x8(%rsp)
   c2fcc:	jmp    c2cc0 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x17f0>
   c2fd1:	mov    $0x5,%r8d
   c2fd7:	mov    $0x12,%r12d
   c2fdd:	lea    -0xa23a8(%rip),%rax        # 20c3c <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.12047894789963178251+0x1dd>
   c2fe4:	mov    %rax,0x8(%rsp)
   c2fe9:	jmp    c2cc0 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x17f0>
   c2fee:	movups 0x58(%rsp),%xmm0
   c2ff3:	mov    %rax,(%rbx)
   c2ff6:	mov    %rcx,0x8(%rbx)
   c2ffa:	mov    %rbp,0x10(%rbx)
   c2ffe:	mov    %r12,0x18(%rbx)
   c3002:	movups %xmm0,0x20(%rbx)
   c3006:	mov    %r14,0x30(%rbx)
   c300a:	mov    0xac(%rsp),%eax
   c3011:	mov    %eax,0x38(%rbx)
   c3014:	movzbl 0x17(%rsp),%eax
   c3019:	mov    %al,0x3c(%rbx)
   c301c:	jmp    c2d27 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1857>
   c3021:	mov    $0x5,%r8d
   c3027:	mov    $0xc,%r12d
   c302d:	lea    -0xa23db(%rip),%rax        # 20c59 <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.12047894789963178251+0x1fa>
   c3034:	mov    %rax,0x8(%rsp)
   c3039:	jmp    c2cc0 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x17f0>
   c303e:	lea    0x1e9503(%rip),%rcx        # 2ac548 <anon.49e524b3d56d2aeb6c463ad9e106202e.63.llvm.12047894789963178251+0x30>
   c3045:	mov    $0x6,%edx
   c304a:	xor    %edi,%edi
   c304c:	call   *0x1f578e(%rip)        # 2b87e0 <_DYNAMIC+0x498>
   c3052:	ud2
   c3054:	mov    0x30(%rsp),%rax
   c3059:	jmp    c2de3 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1913>
   c305e:	mov    $0x10,%r12d
   c3064:	lea    -0xac53b(%rip),%rax        # 16b30 <__abi_tag+0x16834>
   c306b:	jmp    c34cb <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1ffb>
   c3070:	mov    $0x7,%r12d
   c3076:	lea    -0xa24da(%rip),%rax        # 20ba3 <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.12047894789963178251+0x144>
   c307d:	mov    %rax,0x8(%rsp)
   c3082:	xor    %r8d,%r8d
   c3085:	jmp    c2cc0 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x17f0>
   c308a:	lea    -0xa233c(%rip),%rax        # 20d55 <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.12047894789963178251+0x2f6>
   c3091:	jmp    c34cb <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1ffb>
   c3096:	movzbl (%rdi),%eax
   c3099:	lea    -0x41(%rax),%ecx
   c309c:	cmp    $0x1a,%cl
   c309f:	setb   %cl
   c30a2:	shl    $0x5,%cl
   c30a5:	or     %al,%cl
   c30a7:	lea    -0xa2449(%rip),%rdx        # 20c65 <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.12047894789963178251+0x206>
   c30ae:	cmp    $0x65,%cl
   c30b1:	mov    %rdx,0x8(%rsp)
   c30b6:	jne    c3488 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1fb8>
   c30bc:	movzbl 0x1(%rdi),%eax
   c30c0:	lea    -0x41(%rax),%ecx
   c30c3:	cmp    $0x1a,%cl
   c30c6:	setb   %cl
   c30c9:	shl    $0x5,%cl
   c30cc:	or     %al,%cl
   c30ce:	cmp    $0x6e,%cl
   c30d1:	jne    c3488 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1fb8>
   c30d7:	movzbl 0x2(%rdi),%eax
   c30db:	lea    -0x41(%rax),%ecx
   c30de:	cmp    $0x1a,%cl
   c30e1:	setb   %cl
   c30e4:	shl    $0x5,%cl
   c30e7:	or     %al,%cl
   c30e9:	cmp    $0x63,%cl
   c30ec:	jne    c3488 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1fb8>
   c30f2:	movzbl 0x3(%rdi),%eax
   c30f6:	lea    -0x41(%rax),%ecx
   c30f9:	cmp    $0x1a,%cl
   c30fc:	setb   %cl
   c30ff:	shl    $0x5,%cl
   c3102:	or     %al,%cl
   c3104:	cmp    $0x6f,%cl
   c3107:	jne    c3488 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1fb8>
   c310d:	movzbl 0x4(%rdi),%eax
   c3111:	lea    -0x41(%rax),%ecx
   c3114:	cmp    $0x1a,%cl
   c3117:	setb   %cl
   c311a:	shl    $0x5,%cl
   c311d:	or     %al,%cl
   c311f:	cmp    $0x64,%cl
   c3122:	jne    c3488 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1fb8>
   c3128:	movzbl 0x5(%rdi),%eax
   c312c:	lea    -0x41(%rax),%ecx
   c312f:	cmp    $0x1a,%cl
   c3132:	setb   %cl
   c3135:	shl    $0x5,%cl
   c3138:	or     %al,%cl
   c313a:	cmp    $0x69,%cl
   c313d:	jne    c3488 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1fb8>
   c3143:	movzbl 0x6(%rdi),%eax
   c3147:	lea    -0x41(%rax),%ecx
   c314a:	cmp    $0x1a,%cl
   c314d:	setb   %cl
   c3150:	shl    $0x5,%cl
   c3153:	or     %al,%cl
   c3155:	cmp    $0x6e,%cl
   c3158:	jne    c3488 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1fb8>
   c315e:	mov    0x18(%rsp),%rax
   c3163:	movzbl 0x7(%rax),%eax
   c3167:	lea    -0x41(%rax),%ecx
   c316a:	cmp    $0x1a,%cl
   c316d:	setb   %cl
   c3170:	shl    $0x5,%cl
   c3173:	or     %al,%cl
   c3175:	cmp    $0x67,%cl
   c3178:	je     c2ef8 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1a28>
   c317e:	jmp    c3488 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1fb8>
   c3183:	movzbl (%rdi),%eax
   c3186:	lea    -0x41(%rax),%ecx
   c3189:	cmp    $0x1a,%cl
   c318c:	setb   %cl
   c318f:	shl    $0x5,%cl
   c3192:	or     %al,%cl
   c3194:	lea    -0xa2536(%rip),%rdx        # 20c65 <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.12047894789963178251+0x206>
   c319b:	cmp    $0x76,%cl
   c319e:	mov    %rdx,0x8(%rsp)
   c31a3:	jne    c3488 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1fb8>
   c31a9:	movzbl 0x1(%rdi),%eax
   c31ad:	lea    -0x41(%rax),%ecx
   c31b0:	cmp    $0x1a,%cl
   c31b3:	setb   %cl
   c31b6:	shl    $0x5,%cl
   c31b9:	or     %al,%cl
   c31bb:	cmp    $0x65,%cl
   c31be:	jne    c3488 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1fb8>
   c31c4:	movzbl 0x2(%rdi),%eax
   c31c8:	lea    -0x41(%rax),%ecx
   c31cb:	cmp    $0x1a,%cl
   c31ce:	setb   %cl
   c31d1:	shl    $0x5,%cl
   c31d4:	or     %al,%cl
   c31d6:	cmp    $0x72,%cl
   c31d9:	jne    c3488 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1fb8>
   c31df:	movzbl 0x3(%rdi),%eax
   c31e3:	lea    -0x41(%rax),%ecx
   c31e6:	cmp    $0x1a,%cl
   c31e9:	setb   %cl
   c31ec:	shl    $0x5,%cl
   c31ef:	or     %al,%cl
   c31f1:	cmp    $0x73,%cl
   c31f4:	jne    c3488 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1fb8>
   c31fa:	movzbl 0x4(%rdi),%eax
   c31fe:	lea    -0x41(%rax),%ecx
   c3201:	cmp    $0x1a,%cl
   c3204:	setb   %cl
   c3207:	shl    $0x5,%cl
   c320a:	or     %al,%cl
   c320c:	cmp    $0x69,%cl
   c320f:	jne    c3488 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1fb8>
   c3215:	movzbl 0x5(%rdi),%eax
   c3219:	lea    -0x41(%rax),%ecx
   c321c:	cmp    $0x1a,%cl
   c321f:	setb   %cl
   c3222:	shl    $0x5,%cl
   c3225:	or     %al,%cl
   c3227:	cmp    $0x6f,%cl
   c322a:	jne    c3488 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1fb8>
   c3230:	movzbl 0x6(%rdi),%eax
   c3234:	lea    -0x41(%rax),%ecx
   c3237:	cmp    $0x1a,%cl
   c323a:	setb   %cl
   c323d:	shl    $0x5,%cl
   c3240:	or     %al,%cl
   c3242:	cmp    $0x6e,%cl
   c3245:	jne    c3488 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1fb8>
   c324b:	lea    -0xa25e3(%rip),%rax        # 20c6f <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.12047894789963178251+0x210>
   c3252:	mov    %rax,0x8(%rsp)
   c3257:	mov    $0x14,%r12d
   c325d:	mov    $0x2,%r8d
   c3263:	jmp    c2cc0 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x17f0>
   c3268:	movzbl (%rdi),%eax
   c326b:	lea    -0x41(%rax),%ecx
   c326e:	cmp    $0x1a,%cl
   c3271:	setb   %cl
   c3274:	shl    $0x5,%cl
   c3277:	or     %al,%cl
   c3279:	lea    -0xa261b(%rip),%rdx        # 20c65 <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.12047894789963178251+0x206>
   c3280:	cmp    $0x70,%cl
   c3283:	mov    %rdx,0x8(%rsp)
   c3288:	jne    c3488 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1fb8>
   c328e:	movzbl 0x1(%rdi),%eax
   c3292:	lea    -0x41(%rax),%ecx
   c3295:	cmp    $0x1a,%cl
   c3298:	setb   %cl
   c329b:	shl    $0x5,%cl
   c329e:	or     %al,%cl
   c32a0:	cmp    $0x61,%cl
   c32a3:	jne    c3488 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1fb8>
   c32a9:	movzbl 0x2(%rdi),%eax
   c32ad:	lea    -0x41(%rax),%ecx
   c32b0:	cmp    $0x1a,%cl
   c32b3:	setb   %cl
   c32b6:	shl    $0x5,%cl
   c32b9:	or     %al,%cl
   c32bb:	cmp    $0x72,%cl
   c32be:	jne    c3488 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1fb8>
   c32c4:	movzbl 0x3(%rdi),%eax
   c32c8:	lea    -0x41(%rax),%ecx
   c32cb:	cmp    $0x1a,%cl
   c32ce:	setb   %cl
   c32d1:	shl    $0x5,%cl
   c32d4:	or     %al,%cl
   c32d6:	cmp    $0x65,%cl
   c32d9:	jne    c3488 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1fb8>
   c32df:	movzbl 0x4(%rdi),%eax
   c32e3:	lea    -0x41(%rax),%ecx
   c32e6:	cmp    $0x1a,%cl
   c32e9:	setb   %cl
   c32ec:	shl    $0x5,%cl
   c32ef:	or     %al,%cl
   c32f1:	cmp    $0x6e,%cl
   c32f4:	jne    c3488 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1fb8>
   c32fa:	movzbl 0x5(%rdi),%eax
   c32fe:	lea    -0x41(%rax),%ecx
   c3301:	cmp    $0x1a,%cl
   c3304:	setb   %cl
   c3307:	shl    $0x5,%cl
   c330a:	or     %al,%cl
   c330c:	cmp    $0x74,%cl
   c330f:	jne    c3488 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1fb8>
   c3315:	movzbl 0x6(%rdi),%eax
   c3319:	lea    -0x41(%rax),%ecx
   c331c:	cmp    $0x1a,%cl
   c331f:	setb   %cl
   c3322:	shl    $0x5,%cl
   c3325:	or     %al,%cl
   c3327:	cmp    $0x63,%cl
   c332a:	jne    c3488 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1fb8>
   c3330:	mov    0x18(%rsp),%rax
   c3335:	movzbl 0x7(%rax),%eax
   c3339:	lea    -0x41(%rax),%ecx
   c333c:	cmp    $0x1a,%cl
   c333f:	setb   %cl
   c3342:	shl    $0x5,%cl
   c3345:	or     %al,%cl
   c3347:	cmp    $0x69,%cl
   c334a:	jne    c3488 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1fb8>
   c3350:	mov    0x18(%rsp),%rax
   c3355:	movzbl 0x8(%rax),%eax
   c3359:	lea    -0x41(%rax),%ecx
   c335c:	cmp    $0x1a,%cl
   c335f:	setb   %cl
   c3362:	shl    $0x5,%cl
   c3365:	or     %al,%cl
   c3367:	cmp    $0x64,%cl
   c336a:	je     c2ef8 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1a28>
   c3370:	jmp    c3488 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1fb8>
   c3375:	movzbl (%rdi),%eax
   c3378:	lea    -0x41(%rax),%ecx
   c337b:	cmp    $0x1a,%cl
   c337e:	setb   %cl
   c3381:	shl    $0x5,%cl
   c3384:	or     %al,%cl
   c3386:	lea    -0xa2728(%rip),%rdx        # 20c65 <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.12047894789963178251+0x206>
   c338d:	cmp    $0x63,%cl
   c3390:	mov    %rdx,0x8(%rsp)
   c3395:	jne    c3488 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1fb8>
   c339b:	movzbl 0x1(%rdi),%eax
   c339f:	lea    -0x41(%rax),%ecx
   c33a2:	cmp    $0x1a,%cl
   c33a5:	setb   %cl
   c33a8:	shl    $0x5,%cl
   c33ab:	or     %al,%cl
   c33ad:	cmp    $0x72,%cl
   c33b0:	jne    c3488 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1fb8>
   c33b6:	movzbl 0x2(%rdi),%eax
   c33ba:	lea    -0x41(%rax),%ecx
   c33bd:	cmp    $0x1a,%cl
   c33c0:	setb   %cl
   c33c3:	shl    $0x5,%cl
   c33c6:	or     %al,%cl
   c33c8:	cmp    $0x65,%cl
   c33cb:	jne    c3488 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1fb8>
   c33d1:	movzbl 0x3(%rdi),%eax
   c33d5:	lea    -0x41(%rax),%ecx
   c33d8:	cmp    $0x1a,%cl
   c33db:	setb   %cl
   c33de:	shl    $0x5,%cl
   c33e1:	or     %al,%cl
   c33e3:	cmp    $0x61,%cl
   c33e6:	jne    c3488 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1fb8>
   c33ec:	movzbl 0x4(%rdi),%eax
   c33f0:	lea    -0x41(%rax),%ecx
   c33f3:	cmp    $0x1a,%cl
   c33f6:	setb   %cl
   c33f9:	shl    $0x5,%cl
   c33fc:	or     %al,%cl
   c33fe:	cmp    $0x74,%cl
   c3401:	jne    c3488 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1fb8>
   c3407:	movzbl 0x5(%rdi),%eax
   c340b:	lea    -0x41(%rax),%ecx
   c340e:	cmp    $0x1a,%cl
   c3411:	setb   %cl
   c3414:	shl    $0x5,%cl
   c3417:	or     %al,%cl
   c3419:	cmp    $0x65,%cl
   c341c:	jne    c3488 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1fb8>
   c341e:	movzbl 0x6(%rdi),%eax
   c3422:	lea    -0x41(%rax),%ecx
   c3425:	cmp    $0x1a,%cl
   c3428:	setb   %cl
   c342b:	shl    $0x5,%cl
   c342e:	or     %al,%cl
   c3430:	cmp    $0x74,%cl
   c3433:	jne    c3488 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1fb8>
   c3435:	movzbl 0x7(%rdi),%eax
   c3439:	lea    -0x41(%rax),%ecx
   c343c:	cmp    $0x1a,%cl
   c343f:	setb   %cl
   c3442:	shl    $0x5,%cl
   c3445:	or     %al,%cl
   c3447:	cmp    $0x79,%cl
   c344a:	jne    c3488 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1fb8>
   c344c:	mov    0x18(%rsp),%rax
   c3451:	movzbl 0x8(%rax),%eax
   c3455:	lea    -0x41(%rax),%ecx
   c3458:	cmp    $0x1a,%cl
   c345b:	setb   %cl
   c345e:	shl    $0x5,%cl
   c3461:	or     %al,%cl
   c3463:	cmp    $0x70,%cl
   c3466:	jne    c3488 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1fb8>
   c3468:	mov    0x18(%rsp),%rax
   c346d:	movzbl 0x9(%rax),%eax
   c3471:	lea    -0x41(%rax),%ecx
   c3474:	cmp    $0x1a,%cl
   c3477:	setb   %cl
   c347a:	shl    $0x5,%cl
   c347d:	or     %al,%cl
   c347f:	cmp    $0x65,%cl
   c3482:	je     c2ef8 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1a28>
   c3488:	mov    $0x5,%r8d
   c348e:	jmp    c2cc0 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x17f0>
   c3493:	mov    $0x12,%r12d
   c3499:	lea    -0xa28f6(%rip),%rax        # 20baa <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.12047894789963178251+0x14b>
   c34a0:	mov    %rax,0x8(%rsp)
   c34a5:	jmp    c2cc0 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x17f0>
   c34aa:	lea    -0xa2774(%rip),%rax        # 20d3d <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.12047894789963178251+0x2de>
   c34b1:	mov    %rax,0x8(%rsp)
   c34b6:	mov    %rdx,%r8
   c34b9:	jmp    c2cc0 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x17f0>
   c34be:	mov    $0x18,%r12d
   c34c4:	lea    -0xa28a7(%rip),%rax        # 20c24 <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.12047894789963178251+0x1c5>
   c34cb:	mov    %rax,0x8(%rsp)
   c34d0:	mov    $0x2,%r8d
   c34d6:	jmp    c2cc0 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x17f0>
   c34db:	mov    $0x10,%r12d
   c34e1:	lea    -0xac498(%rip),%rax        # 17050 <anon.83d0f48a2d0b5ff7b6e24226a84ebc25.25.llvm.569205349683904225+0x20>
   c34e8:	jmp    c307d <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1bad>
   c34ed:	mov    $0x8,%r8d
   c34f3:	mov    $0xc,%r12d
   c34f9:	lea    -0xa2869(%rip),%rax        # 20c97 <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.12047894789963178251+0x238>
   c3500:	mov    %rax,0x8(%rsp)
   c3505:	jmp    c2cc0 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x17f0>
   c350a:	movzbl 0x17(%rsp),%eax
   c350f:	cmp    $0x2,%eax
   c3512:	jae    c356c <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x209c>
   c3514:	mov    $0x28,%r12d
   c351a:	lea    -0xa2857(%rip),%rax        # 20cca <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.12047894789963178251+0x26b>
   c3521:	jmp    c355c <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x208c>
   c3523:	mov    $0xc,%r12d
   c3529:	lea    -0xa2899(%rip),%rax        # 20c97 <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.12047894789963178251+0x238>
   c3530:	jmp    c355c <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x208c>
   c3532:	mov    $0x1a,%r12d
   c3538:	mov    $0x2,%edx
   c353d:	lea    -0xa27a5(%rip),%rax        # 20d9f <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.12047894789963178251+0x340>
   c3544:	jmp    c34b1 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1fe1>
   c3549:	lea    -0xa27d2(%rip),%rax        # 20d7e <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.12047894789963178251+0x31f>
   c3550:	jmp    c34cb <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1ffb>
   c3555:	lea    -0xa28b9(%rip),%rax        # 20ca3 <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.12047894789963178251+0x244>
   c355c:	mov    %rax,0x8(%rsp)
   c3561:	mov    $0x8,%r8d
   c3567:	jmp    c2cc0 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x17f0>
   c356c:	mov    $0x4,%r8d
   c3572:	mov    $0x19,%r12d
   c3578:	lea    -0xa28ce(%rip),%rax        # 20cb1 <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.12047894789963178251+0x252>
   c357f:	mov    %rax,0x8(%rsp)
   c3584:	jmp    c2cc0 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x17f0>
   c3589:	lea    -0xa2861(%rip),%rax        # 20d2f <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.12047894789963178251+0x2d0>
   c3590:	jmp    c307d <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x1bad>
   c3595:	mov    0x20(%rsp),%rax
   c359a:	mov    %rax,0x8(%rsp)
   c359f:	mov    $0x7,%r8d
   c35a5:	jmp    c2cc0 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x17f0>
   c35aa:	mov    $0x1a,%r12d
   c35b0:	lea    -0xa28c5(%rip),%rax        # 20cf2 <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.12047894789963178251+0x293>
   c35b7:	jmp    c355c <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x208c>
   c35b9:	mov    $0x23,%r12d
   c35bf:	lea    -0xa28ba(%rip),%rax        # 20d0c <anon.49e524b3d56d2aeb6c463ad9e106202e.61.llvm.12047894789963178251+0x2ad>
   c35c6:	jmp    c355c <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x208c>
   c35c8:	mov    0x30(%rsp),%rax
   c35cd:	jmp    c359a <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x20ca>
   c35cf:	mov    $0x7,%r8d
   c35d5:	jmp    c2cc0 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x17f0>
   c35da:	jmp    c35de <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x210e>
   c35dc:	jmp    c35de <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x210e>
   c35de:	mov    %rax,%rbx
   c35e1:	mov    0x50(%rsp),%rsi
   c35e6:	test   %rsi,%rsi
   c35e9:	jne    c35fd <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x212d>
   c35eb:	mov    0x78(%rsp),%rax
   c35f0:	test   %rax,%rax
   c35f3:	jne    c361b <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x214b>
   c35f5:	mov    %rbx,%rdi
   c35f8:	call   2aa1b0 <_Unwind_Resume@plt>
   c35fd:	mov    0x58(%rsp),%rdi
   c3602:	shl    $0x5,%rsi
   c3606:	mov    $0x8,%edx
   c360b:	call   *0x1f4f87(%rip)        # 2b8598 <_DYNAMIC+0x250>
   c3611:	mov    0x78(%rsp),%rax
   c3616:	test   %rax,%rax
   c3619:	je     c35f5 <rvvdk_vmdk::descriptor::Descriptor::parse_with_limits+0x2125>
   c361b:	mov    0x80(%rsp),%rdi
   c3623:	imul   $0x38,%rax,%rsi
   c3627:	mov    $0x8,%edx
   c362c:	call   *0x1f4f66(%rip)        # 2b8598 <_DYNAMIC+0x250>
   c3632:	mov    %rbx,%rdi
   c3635:	call   2aa1b0 <_Unwind_Resume@plt>
