
target/r511b/reference/map-candidate:     file format elf64-x86-64


Disassembly of section .text:

00000000000b6700 <rvvdk_vmdk::stream_map::StreamMap::read_from>:
   b6700:	push   %rbp
   b6701:	push   %r15
   b6703:	push   %r14
   b6705:	push   %r13
   b6707:	push   %r12
   b6709:	push   %rbx
   b670a:	sub    $0x1000,%rsp
   b6711:	movq   $0x0,(%rsp)
   b6719:	sub    $0x5b8,%rsp
   b6720:	mov    %r8,%rbp
   b6723:	mov    %rdi,%r15
   b6726:	mov    %rsi,0x240(%rsp)
   b672e:	mov    %rdx,0x248(%rsp)
   b6736:	mov    %rcx,0x88(%rsp)
   b673e:	movups (%r8),%xmm0
   b6742:	movups 0x10(%r8),%xmm1
   b6747:	movups 0x20(%r8),%xmm2
   b674c:	movups 0x30(%r8),%xmm3
   b6751:	movaps %xmm2,0x570(%rsp)
   b6759:	movaps %xmm0,0x550(%rsp)
   b6761:	movaps %xmm1,0x560(%rsp)
   b6769:	movaps %xmm3,0x580(%rsp)
   b6771:	movups 0x40(%r8),%xmm0
   b6776:	movaps %xmm0,0x590(%rsp)
   b677e:	mov    0x50(%r8),%rax
   b6782:	mov    %rax,0x5a0(%rsp)
   b678a:	mov    0x578(%rsp),%rax
   b6792:	mov    0x60(%r8),%rbx
   b6796:	cmp    %rax,%rbx
   b6799:	cmovb  %rbx,%rax
   b679d:	mov    %rax,0x578(%rsp)
   b67a5:	lea    0x5b0(%rsp),%rdi
   b67ad:	lea    0x240(%rsp),%r13
   b67b5:	lea    0x550(%rsp),%rax
   b67bd:	mov    %r13,%rsi
   b67c0:	mov    %rcx,%rdx
   b67c3:	mov    %rax,%rcx
   b67c6:	call   b12b0 <rvvdk_vmdk::stream::StreamEnvelope::read_from>
   b67cb:	mov    0x5b0(%rsp),%r14
   b67d3:	movups 0x5b8(%rsp),%xmm0
   b67db:	movaps %xmm0,0xdb0(%rsp)
   b67e3:	movups 0x5c8(%rsp),%xmm0
   b67eb:	movaps %xmm0,0xdc0(%rsp)
   b67f3:	cmp    $0x3,%r14
   b67f7:	jne    b681f <rvvdk_vmdk::stream_map::StreamMap::read_from+0x11f>
   b67f9:	movaps 0xdb0(%rsp),%xmm0
   b6801:	movaps 0xdc0(%rsp),%xmm1
   b6809:	movups %xmm1,0x18(%r15)
   b680e:	movups %xmm0,0x8(%r15)
   b6813:	movq   $0x3,(%r15)
   b681a:	jmp    b6f65 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x865>
   b681f:	mov    0x608(%rsp),%rax
   b6827:	mov    %rax,0x1b0(%rsp)
   b682f:	movups 0x5d8(%rsp),%xmm0
   b6837:	movups 0x5e8(%rsp),%xmm1
   b683f:	movups 0x5f8(%rsp),%xmm2
   b6847:	movaps %xmm2,0x1a0(%rsp)
   b684f:	movaps %xmm1,0x190(%rsp)
   b6857:	movaps %xmm0,0x180(%rsp)
   b685f:	mov    0x620(%rsp),%r12
   b6867:	mov    0x628(%rsp),%rax
   b686f:	mov    0x628(%rsp),%edx
   b6876:	movups 0x610(%rsp),%xmm0
   b687e:	mov    0x610(%rsp),%rsi
   b6886:	movaps 0xdb0(%rsp),%xmm1
   b688e:	movaps 0xdc0(%rsp),%xmm2
   b6896:	movaps %xmm1,0x330(%rsp)
   b689e:	movaps %xmm2,0x340(%rsp)
   b68a6:	mov    %r14,0x250(%rsp)
   b68ae:	movaps 0xdb0(%rsp),%xmm1
   b68b6:	movaps 0xdc0(%rsp),%xmm2
   b68be:	movups %xmm2,0x268(%rsp)
   b68c6:	movups %xmm1,0x258(%rsp)
   b68ce:	mov    0x1b0(%rsp),%rcx
   b68d6:	mov    %rcx,0x2a8(%rsp)
   b68de:	movaps 0x180(%rsp),%xmm1
   b68e6:	movaps 0x190(%rsp),%xmm2
   b68ee:	movaps 0x1a0(%rsp),%xmm3
   b68f6:	movups %xmm3,0x298(%rsp)
   b68fe:	movups %xmm2,0x288(%rsp)
   b6906:	movups %xmm1,0x278(%rsp)
   b690e:	movups %xmm0,0x2b0(%rsp)
   b6916:	mov    %r12,0x2c0(%rsp)
   b691e:	mov    %rax,0x2c8(%rsp)
   b6926:	mov    0x278(%rsp),%rcx
   b692e:	movabs $0x1000000010000,%rax
   b6938:	cmp    %rax,%rcx
   b693b:	jb     b6964 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x264>
   b693d:	movq   $0xb,0x8(%r15)
   b6945:	lea    -0x962db(%rip),%rax        # 20671 <anon.c1dd2dbd4cf59c2854d815f601d2b4b9.42.llvm.4207380642161847320+0x88>
   b694c:	mov    %rax,0x10(%r15)
   b6950:	movq   $0xe,0x18(%r15)
   b6958:	movq   $0x3,(%r15)
   b695f:	jmp    b6f65 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x865>
   b6964:	mov    %rsi,0x80(%rsp)
   b696c:	mov    %edx,0xe4(%rsp)
   b6973:	mov    %rcx,%rdx
   b6976:	shr    $0x19,%rdx
   b697a:	mov    %rcx,0xd0(%rsp)
   b6982:	mov    %ecx,%eax
   b6984:	and    $0x1ff0000,%eax
   b6989:	cmp    $0x1,%rax
   b698d:	sbb    $0xffffffffffffffff,%rdx
   b6991:	mov    %rdx,0x78(%rsp)
   b6996:	mov    0x258(%rsp),%rax
   b699e:	mov    %rax,0x8(%rsp)
   b69a3:	mov    0x260(%rsp),%rax
   b69ab:	mov    %rax,0x30(%rsp)
   b69b0:	xor    %eax,%eax
   b69b2:	cmp    $0x2,%r14
   b69b6:	cmovne %r14,%rax
   b69ba:	xor    %edx,%edx
   b69bc:	mov    %rax,0x10(%rsp)
   b69c1:	cmp    $0x1,%rax
   b69c5:	sete   %dl
   b69c8:	inc    %rdx
   b69cb:	mov    0x2a0(%rsp),%rsi
   b69d3:	lea    0x5b0(%rsp),%rdi
   b69db:	mov    %rsi,(%rsp)
   b69df:	mov    %rdx,0x20(%rsp)
   b69e4:	call   *0x2106be(%rip)        # 2c70a8 <_DYNAMIC+0x618>
   b69ea:	mov    0x5b0(%rsp),%rax
   b69f2:	mov    0x5b8(%rsp),%rcx
   b69fa:	cmp    $0x10,%rax
   b69fe:	jne    b6a30 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x330>
   b6a00:	mov    0x58(%rbp),%rax
   b6a04:	cmp    %rax,%rcx
   b6a07:	jbe    b6a51 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x351>
   b6a09:	movq   $0xb,0x8(%r15)
   b6a11:	lea    -0x9f718(%rip),%rax        # 17300 <anon.d625489d584c397ac22d75864c33158f.35.llvm.5936746164385555759+0x1a0>
   b6a18:	mov    %rax,0x10(%r15)
   b6a1c:	movq   $0x10,0x18(%r15)
   b6a24:	movq   $0x3,(%r15)
   b6a2b:	jmp    b6f65 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x865>
   b6a30:	movups 0x5c0(%rsp),%xmm0
   b6a38:	movups %xmm0,0x18(%r15)
   b6a3d:	mov    %rax,0x8(%r15)
   b6a41:	mov    %rcx,0x10(%r15)
   b6a45:	movq   $0x3,(%r15)
   b6a4c:	jmp    b6f65 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x865>
   b6a51:	mov    %rcx,0x58(%rsp)
   b6a56:	mov    %rax,0xa0(%rsp)
   b6a5e:	mov    %r14,0x98(%rsp)
   b6a66:	mov    $0x1,%eax
   b6a6b:	mov    %rax,0x40(%rsp)
   b6a70:	mov    %rax,0x18(%rsp)
   b6a75:	mov    (%rsp),%r14
   b6a79:	test   %r14,%r14
   b6a7c:	jne    b6c5d <rvvdk_vmdk::stream_map::StreamMap::read_from+0x55d>
   b6a82:	xor    %eax,%eax
   b6a84:	cmpl   $0x1,0x10(%rsp)
   b6a89:	mov    $0x0,%ecx
   b6a8e:	cmove  %r14,%rcx
   b6a92:	test   %rcx,%rcx
   b6a95:	mov    %rcx,0x60(%rsp)
   b6a9a:	jne    b6ca6 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x5a6>
   b6aa0:	mov    %rax,0x50(%rsp)
   b6aa5:	mov    %r13,0x100(%rsp)
   b6aad:	mov    0x88(%rsp),%rax
   b6ab5:	mov    %rax,0x108(%rsp)
   b6abd:	mov    %r12,0x110(%rsp)
   b6ac5:	mov    %rbx,0x118(%rsp)
   b6acd:	lea    0x5b0(%rsp),%rdi
   b6ad5:	mov    %r14,%rsi
   b6ad8:	mov    0x20(%rsp),%rdx
   b6add:	mov    %r14,%r13
   b6ae0:	mov    0x18(%rsp),%r14
   b6ae5:	call   *0x2105bd(%rip)        # 2c70a8 <_DYNAMIC+0x618>
   b6aeb:	mov    0x5b0(%rsp),%rax
   b6af3:	mov    0x5b8(%rsp),%rsi
   b6afb:	cmp    $0x10,%rax
   b6aff:	jne    b6f19 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x819>
   b6b05:	lea    0xdb0(%rsp),%rdi
   b6b0d:	mov    $0x200,%edx
   b6b12:	call   *0x210588(%rip)        # 2c70a0 <_DYNAMIC+0x610>
   b6b18:	mov    0xdb0(%rsp),%rax
   b6b20:	mov    0xdb8(%rsp),%rdx
   b6b28:	cmp    $0x10,%rax
   b6b2c:	jne    b6b7a <rvvdk_vmdk::stream_map::StreamMap::read_from+0x47a>
   b6b2e:	lea    0x350(%rsp),%rdi
   b6b36:	mov    %r12,%rsi
   b6b39:	call   *0x210561(%rip)        # 2c70a0 <_DYNAMIC+0x610>
   b6b3f:	mov    0x350(%rsp),%rcx
   b6b47:	mov    0x358(%rsp),%rax
   b6b4f:	cmp    $0x10,%rcx
   b6b53:	jne    b6b94 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x494>
   b6b55:	cmp    %rbx,%rax
   b6b58:	jbe    b6bae <rvvdk_vmdk::stream_map::StreamMap::read_from+0x4ae>
   b6b5a:	movq   $0xb,0x8(%r15)
   b6b62:	lea    -0x96506(%rip),%rax        # 20663 <anon.c1dd2dbd4cf59c2854d815f601d2b4b9.42.llvm.4207380642161847320+0x7a>
   b6b69:	mov    %rax,0x10(%r15)
   b6b6d:	movq   $0xe,0x18(%r15)
   b6b75:	jmp    b6f2e <rvvdk_vmdk::stream_map::StreamMap::read_from+0x82e>
   b6b7a:	movups 0xdc0(%rsp),%xmm0
   b6b82:	movups %xmm0,0x18(%r15)
   b6b87:	mov    %rax,0x8(%r15)
   b6b8b:	mov    %rdx,0x10(%r15)
   b6b8f:	jmp    b6f2e <rvvdk_vmdk::stream_map::StreamMap::read_from+0x82e>
   b6b94:	movups 0x360(%rsp),%xmm0
   b6b9c:	movups %xmm0,0x18(%r15)
   b6ba1:	mov    %rcx,0x8(%r15)
   b6ba5:	mov    %rax,0x10(%r15)
   b6ba9:	jmp    b6f2e <rvvdk_vmdk::stream_map::StreamMap::read_from+0x82e>
   b6bae:	lea    0x5b0(%rsp),%rdi
   b6bb6:	lea    0x100(%rsp),%rsi
   b6bbe:	mov    0x80(%rsp),%rdx
   b6bc6:	mov    %r14,%rcx
   b6bc9:	mov    %r13,%r8
   b6bcc:	call   b65a0 <rvvdk_vmdk::stream_map::Budget<S>::read>
   b6bd1:	cmpl   $0x10,0x5b0(%rsp)
   b6bd9:	jne    b6cf8 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x5f8>
   b6bdf:	cmpq   $0x1,0x10(%rsp)
   b6be5:	jne    b6c19 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x519>
   b6be7:	lea    0x5b0(%rsp),%rdi
   b6bef:	lea    0x100(%rsp),%rsi
   b6bf7:	mov    0x8(%rsp),%rdx
   b6bfc:	mov    0x40(%rsp),%rcx
   b6c01:	mov    0x50(%rsp),%r8
   b6c06:	call   b65a0 <rvvdk_vmdk::stream_map::Budget<S>::read>
   b6c0b:	cmpl   $0x10,0x5b0(%rsp)
   b6c13:	jne    b6cf8 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x5f8>
   b6c19:	mov    0x78(%rsp),%rax
   b6c1e:	lea    0x0(,%rax,4),%rdi
   b6c26:	cmp    %r13,%rdi
   b6c29:	mov    0x50(%rsp),%rsi
   b6c2e:	ja     b7353 <rvvdk_vmdk::stream_map::StreamMap::read_from+0xc53>
   b6c34:	shrq   $0x10,0xd0(%rsp)
   b6c3d:	lea    (%r14,%rdi,1),%rax
   b6c41:	mov    %rdi,%rcx
   b6c44:	cmp    %rcx,%r13
   b6c47:	je     b6d9e <rvvdk_vmdk::stream_map::StreamMap::read_from+0x69e>
   b6c4d:	cmpb   $0x0,(%r14,%rcx,1)
   b6c52:	lea    0x1(%rcx),%rcx
   b6c56:	je     b6c44 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x544>
   b6c58:	jmp    b6dc3 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x6c3>
   b6c5d:	lea    0x5b0(%rsp),%rdi
   b6c65:	mov    $0x1,%edx
   b6c6a:	mov    $0x1,%r8d
   b6c70:	mov    $0x1,%r9d
   b6c76:	xor    %esi,%esi
   b6c78:	mov    %r14,%rcx
   b6c7b:	call   cead0 <alloc::raw_vec::RawVecInner<A>::finish_grow>
   b6c80:	cmpb   $0x0,0x5b0(%rsp)
   b6c88:	je     b6d17 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x617>
   b6c8e:	mov    %r14,0x10(%r15)
   b6c92:	movq   $0xd,0x8(%r15)
   b6c9a:	movq   $0x3,(%r15)
   b6ca1:	jmp    b6f65 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x865>
   b6ca6:	lea    0x5b0(%rsp),%rdi
   b6cae:	mov    $0x1,%edx
   b6cb3:	mov    $0x1,%r8d
   b6cb9:	mov    $0x1,%r9d
   b6cbf:	xor    %esi,%esi
   b6cc1:	mov    %rcx,%r14
   b6cc4:	call   cead0 <alloc::raw_vec::RawVecInner<A>::finish_grow>
   b6cc9:	cmpb   $0x0,0x5b0(%rsp)
   b6cd1:	je     b6d58 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x658>
   b6cd7:	mov    %r14,0x10(%r15)
   b6cdb:	movq   $0xd,0x8(%r15)
   b6ce3:	movq   $0x3,(%r15)
   b6cea:	mov    (%rsp),%r13
   b6cee:	mov    0x18(%rsp),%r14
   b6cf3:	jmp    b6f4f <rvvdk_vmdk::stream_map::StreamMap::read_from+0x84f>
   b6cf8:	movups 0x5b0(%rsp),%xmm0
   b6d00:	movups 0x5c0(%rsp),%xmm1
   b6d08:	movups %xmm1,0x18(%r15)
   b6d0d:	movups %xmm0,0x8(%r15)
   b6d12:	jmp    b6f2e <rvvdk_vmdk::stream_map::StreamMap::read_from+0x82e>
   b6d17:	mov    0x5b8(%rsp),%rax
   b6d1f:	mov    %rax,0x18(%rsp)
   b6d24:	cmp    $0x1,%r14
   b6d28:	je     b6d4c <rvvdk_vmdk::stream_map::StreamMap::read_from+0x64c>
   b6d2a:	mov    (%rsp),%rax
   b6d2e:	lea    -0x1(%rax),%rdx
   b6d32:	mov    0x18(%rsp),%r14
   b6d37:	mov    %r14,%rdi
   b6d3a:	xor    %esi,%esi
   b6d3c:	call   *0x2101d6(%rip)        # 2c6f18 <memset@GLIBC_2.2.5>
   b6d42:	mov    (%rsp),%rax
   b6d46:	add    %r14,%rax
   b6d49:	dec    %rax
   b6d4c:	movb   $0x0,(%rax)
   b6d4f:	mov    (%rsp),%r14
   b6d53:	jmp    b6a82 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x382>
   b6d58:	mov    0x5b8(%rsp),%rax
   b6d60:	mov    %rax,0x40(%rsp)
   b6d65:	cmp    $0x1,%r14
   b6d69:	je     b6d8f <rvvdk_vmdk::stream_map::StreamMap::read_from+0x68f>
   b6d6b:	mov    0x60(%rsp),%rax
   b6d70:	lea    -0x1(%rax),%rdx
   b6d74:	mov    0x40(%rsp),%r14
   b6d79:	mov    %r14,%rdi
   b6d7c:	xor    %esi,%esi
   b6d7e:	call   *0x210194(%rip)        # 2c6f18 <memset@GLIBC_2.2.5>
   b6d84:	mov    0x60(%rsp),%rax
   b6d89:	add    %r14,%rax
   b6d8c:	dec    %rax
   b6d8f:	movb   $0x0,(%rax)
   b6d92:	mov    (%rsp),%r14
   b6d96:	mov    %r14,%rax
   b6d99:	jmp    b6aa0 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x3a0>
   b6d9e:	test   %rsi,%rsi
   b6da1:	je     b6de3 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x6e3>
   b6da3:	cmp    %rsi,%rdi
   b6da6:	ja     b75bf <rvvdk_vmdk::stream_map::StreamMap::read_from+0xebf>
   b6dac:	mov    %rdi,%rcx
   b6daf:	mov    0x40(%rsp),%rdx
   b6db4:	cmp    %rcx,%rsi
   b6db7:	je     b6de3 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x6e3>
   b6db9:	cmpb   $0x0,(%rdx,%rcx,1)
   b6dbd:	lea    0x1(%rcx),%rcx
   b6dc1:	je     b6db4 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x6b4>
   b6dc3:	movq   $0xa,0x8(%r15)
   b6dcb:	lea    -0x965cf(%rip),%rax        # 20803 <anon.c1dd2dbd4cf59c2854d815f601d2b4b9.42.llvm.4207380642161847320+0x21a>
   b6dd2:	mov    %rax,0x10(%r15)
   b6dd6:	movq   $0x16,0x18(%r15)
   b6dde:	jmp    b6f2e <rvvdk_vmdk::stream_map::StreamMap::read_from+0x82e>
   b6de3:	mov    %r14,0x5b0(%rsp)
   b6deb:	mov    %rdi,0x5b8(%rsp)
   b6df3:	mov    %rax,0x5c0(%rsp)
   b6dfb:	movq   $0x0,0x5c8(%rsp)
   b6e07:	movq   $0x4,0x5d0(%rsp)
   b6e13:	lea    0x5b0(%rsp),%rdi
   b6e1b:	call   b61a0 <<core::iter::adapters::filter::Filter<I,P> as core::iter::traits::iterator::Iterator>::count>
   b6e20:	mov    %rax,0x38(%rsp)
   b6e25:	lea    0x5b0(%rsp),%rdi
   b6e2d:	mov    0x38(%rsp),%rsi
   b6e32:	mov    0x20(%rsp),%rdx
   b6e37:	call   *0x21026b(%rip)        # 2c70a8 <_DYNAMIC+0x618>
   b6e3d:	mov    0x5b0(%rsp),%rax
   b6e45:	mov    0x5b8(%rsp),%rsi
   b6e4d:	cmp    $0x10,%rax
   b6e51:	jne    b6f19 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x819>
   b6e57:	lea    0xdb0(%rsp),%rdi
   b6e5f:	mov    $0x4,%edx
   b6e64:	call   *0x210236(%rip)        # 2c70a0 <_DYNAMIC+0x610>
   b6e6a:	mov    0xdb0(%rsp),%rax
   b6e72:	mov    0xdb8(%rsp),%r12
   b6e7a:	cmp    $0x10,%rax
   b6e7e:	jne    b6f7a <rvvdk_vmdk::stream_map::StreamMap::read_from+0x87a>
   b6e84:	lea    0x5b0(%rsp),%rdi
   b6e8c:	mov    $0x10,%edx
   b6e91:	mov    %r12,%rsi
   b6e94:	call   *0x21020e(%rip)        # 2c70a8 <_DYNAMIC+0x618>
   b6e9a:	mov    0x5b0(%rsp),%rax
   b6ea2:	mov    0x5b8(%rsp),%rdx
   b6eaa:	cmp    $0x10,%rax
   b6eae:	jne    b6f91 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x891>
   b6eb4:	lea    0xdb0(%rsp),%rdi
   b6ebc:	mov    0x58(%rsp),%rsi
   b6ec1:	call   *0x2101d9(%rip)        # 2c70a0 <_DYNAMIC+0x610>
   b6ec7:	mov    0xdb0(%rsp),%rax
   b6ecf:	mov    0xdb8(%rsp),%rcx
   b6ed7:	mov    %rcx,0x58(%rsp)
   b6edc:	cmp    $0x10,%rax
   b6ee0:	jne    b6f9e <rvvdk_vmdk::stream_map::StreamMap::read_from+0x89e>
   b6ee6:	mov    0x58(%rsp),%rax
   b6eeb:	cmp    0xa0(%rsp),%rax
   b6ef3:	jbe    b6fdb <rvvdk_vmdk::stream_map::StreamMap::read_from+0x8db>
   b6ef9:	movq   $0xb,0x8(%r15)
   b6f01:	lea    -0x9fc08(%rip),%rax        # 17300 <anon.d625489d584c397ac22d75864c33158f.35.llvm.5936746164385555759+0x1a0>
   b6f08:	mov    %rax,0x10(%r15)
   b6f0c:	movq   $0x10,0x18(%r15)
   b6f14:	jmp    b6fb8 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x8b8>
   b6f19:	movups 0x5c0(%rsp),%xmm0
   b6f21:	movups %xmm0,0x18(%r15)
   b6f26:	mov    %rax,0x8(%r15)
   b6f2a:	mov    %rsi,0x10(%r15)
   b6f2e:	movq   $0x3,(%r15)
   b6f35:	mov    0x60(%rsp),%rsi
   b6f3a:	test   %rsi,%rsi
   b6f3d:	je     b6f4f <rvvdk_vmdk::stream_map::StreamMap::read_from+0x84f>
   b6f3f:	mov    $0x1,%edx
   b6f44:	mov    0x40(%rsp),%rdi
   b6f49:	call   *0x20fd91(%rip)        # 2c6ce0 <_DYNAMIC+0x250>
   b6f4f:	test   %r13,%r13
   b6f52:	je     b6f65 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x865>
   b6f54:	mov    $0x1,%edx
   b6f59:	mov    %r14,%rdi
   b6f5c:	mov    %r13,%rsi
   b6f5f:	call   *0x20fd7b(%rip)        # 2c6ce0 <_DYNAMIC+0x250>
   b6f65:	mov    %r15,%rax
   b6f68:	add    $0x15b8,%rsp
   b6f6f:	pop    %rbx
   b6f70:	pop    %r12
   b6f72:	pop    %r13
   b6f74:	pop    %r14
   b6f76:	pop    %r15
   b6f78:	pop    %rbp
   b6f79:	ret
   b6f7a:	movups 0xdc0(%rsp),%xmm0
   b6f82:	movups %xmm0,0x18(%r15)
   b6f87:	mov    %rax,0x8(%r15)
   b6f8b:	mov    %r12,0x10(%r15)
   b6f8f:	jmp    b6f2e <rvvdk_vmdk::stream_map::StreamMap::read_from+0x82e>
   b6f91:	movups 0x5c0(%rsp),%xmm0
   b6f99:	jmp    b6b82 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x482>
   b6f9e:	movups 0xdc0(%rsp),%xmm0
   b6fa6:	movups %xmm0,0x18(%r15)
   b6fab:	mov    %rax,0x8(%r15)
   b6faf:	mov    0x58(%rsp),%rax
   b6fb4:	mov    %rax,0x10(%r15)
   b6fb8:	movq   $0x3,(%r15)
   b6fbf:	mov    (%rsp),%r13
   b6fc3:	mov    0x18(%rsp),%r14
   b6fc8:	mov    0x60(%rsp),%rsi
   b6fcd:	test   %rsi,%rsi
   b6fd0:	jne    b6f3f <rvvdk_vmdk::stream_map::StreamMap::read_from+0x83f>
   b6fd6:	jmp    b6f4f <rvvdk_vmdk::stream_map::StreamMap::read_from+0x84f>
   b6fdb:	lea    0x5b0(%rsp),%rdi
   b6fe3:	mov    %r12,%rsi
   b6fe6:	call   b8730 <rvvdk_vmdk::stream_map::allocated>
   b6feb:	mov    0x5b0(%rsp),%rax
   b6ff3:	movups 0x5b8(%rsp),%xmm0
   b6ffb:	movaps %xmm0,0xdb0(%rsp)
   b7003:	mov    0x5c8(%rsp),%rcx
   b700b:	mov    %rcx,0xdc0(%rsp)
   b7013:	cmp    $0x10,%rax
   b7017:	jne    b7178 <rvvdk_vmdk::stream_map::StreamMap::read_from+0xa78>
   b701d:	mov    0xdc0(%rsp),%rax
   b7025:	mov    %rax,0xc0(%rsp)
   b702d:	movaps 0xdb0(%rsp),%xmm0
   b7035:	movaps %xmm0,0xb0(%rsp)
   b703d:	movq   $0x0,0xd8(%rsp)
   b7049:	lea    0x88(%rsp),%rax
   b7051:	mov    %rax,0xe8(%rsp)
   b7059:	lea    0xb0(%rsp),%rax
   b7061:	mov    %rax,0xf0(%rsp)
   b7069:	lea    0xd8(%rsp),%rax
   b7071:	mov    %rax,0xf8(%rsp)
   b7079:	lea    0x5b0(%rsp),%rdi
   b7081:	lea    0xe8(%rsp),%rsi
   b7089:	mov    $0x200,%ecx
   b708e:	xor    %edx,%edx
   b7090:	call   b8670 <rvvdk_vmdk::stream_map::StreamMap::read_from::{{closure}}>
   b7095:	cmpl   $0x10,0x5b0(%rsp)
   b709d:	jne    b74e9 <rvvdk_vmdk::stream_map::StreamMap::read_from+0xde9>
   b70a3:	mov    0x288(%rsp),%rdx
   b70ab:	mov    0x290(%rsp),%rcx
   b70b3:	lea    0x5b0(%rsp),%rdi
   b70bb:	lea    0xe8(%rsp),%rsi
   b70c3:	call   b8670 <rvvdk_vmdk::stream_map::StreamMap::read_from::{{closure}}>
   b70c8:	cmpl   $0x10,0x5b0(%rsp)
   b70d0:	jne    b74e9 <rvvdk_vmdk::stream_map::StreamMap::read_from+0xde9>
   b70d6:	lea    0x5b0(%rsp),%rdi
   b70de:	lea    0xe8(%rsp),%rsi
   b70e6:	mov    0x80(%rsp),%rdx
   b70ee:	mov    (%rsp),%rcx
   b70f2:	call   b8670 <rvvdk_vmdk::stream_map::StreamMap::read_from::{{closure}}>
   b70f7:	cmpl   $0x10,0x5b0(%rsp)
   b70ff:	jne    b74e9 <rvvdk_vmdk::stream_map::StreamMap::read_from+0xde9>
   b7105:	testb  $0x1,0x10(%rsp)
   b710a:	je     b7139 <rvvdk_vmdk::stream_map::StreamMap::read_from+0xa39>
   b710c:	lea    0x5b0(%rsp),%rdi
   b7114:	lea    0xe8(%rsp),%rsi
   b711c:	mov    0x8(%rsp),%rdx
   b7121:	mov    0x30(%rsp),%rcx
   b7126:	call   b8670 <rvvdk_vmdk::stream_map::StreamMap::read_from::{{closure}}>
   b712b:	cmpl   $0x10,0x5b0(%rsp)
   b7133:	jne    b74e9 <rvvdk_vmdk::stream_map::StreamMap::read_from+0xde9>
   b7139:	cmpq   $0x0,0x78(%rsp)
   b713f:	je     b7537 <rvvdk_vmdk::stream_map::StreamMap::read_from+0xe37>
   b7145:	mov    0x298(%rsp),%rax
   b714d:	mov    %rax,0x8(%rsp)
   b7152:	cmpl   $0x2,0x98(%rsp)
   b715a:	jne    b7364 <rvvdk_vmdk::stream_map::StreamMap::read_from+0xc64>
   b7160:	mov    0x80(%rsp),%rax
   b7168:	add    $0xfffffffffffffe00,%rax
   b716e:	mov    %rax,0x28(%rsp)
   b7173:	xor    %r14d,%r14d
   b7176:	jmp    b71ae <rvvdk_vmdk::stream_map::StreamMap::read_from+0xaae>
   b7178:	mov    0xdc0(%rsp),%rcx
   b7180:	mov    %rcx,0x20(%r15)
   b7184:	movaps 0xdb0(%rsp),%xmm0
   b718c:	movups %xmm0,0x10(%r15)
   b7191:	mov    %rax,0x8(%r15)
   b7195:	jmp    b6fb8 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x8b8>
   b719a:	mov    %r14,%rax
   b719d:	inc    %rax
   b71a0:	mov    %rax,%r14
   b71a3:	cmp    %rax,0x78(%rsp)
   b71a8:	je     b7537 <rvvdk_vmdk::stream_map::StreamMap::read_from+0xe37>
   b71ae:	mov    0x18(%rsp),%rdi
   b71b3:	mov    (%rsp),%rsi
   b71b7:	mov    %r14,%rdx
   b71ba:	call   *0x20fed8(%rip)        # 2c7098 <_DYNAMIC+0x608>
   b71c0:	mov    %eax,%r13d
   b71c3:	xor    %r12d,%r12d
   b71c6:	cmpl   $0x1,0x10(%rsp)
   b71cb:	jne    b71f7 <rvvdk_vmdk::stream_map::StreamMap::read_from+0xaf7>
   b71cd:	mov    0x40(%rsp),%rdi
   b71d2:	mov    0x50(%rsp),%rsi
   b71d7:	mov    %r14,%rdx
   b71da:	call   *0x20feb8(%rip)        # 2c7098 <_DYNAMIC+0x608>
   b71e0:	mov    %eax,%r12d
   b71e3:	test   %r13d,%r13d
   b71e6:	sete   %al
   b71e9:	test   %r12d,%r12d
   b71ec:	sete   %cl
   b71ef:	xor    %al,%cl
   b71f1:	jne    b7691 <rvvdk_vmdk::stream_map::StreamMap::read_from+0xf91>
   b71f7:	test   %r13d,%r13d
   b71fa:	je     b7210 <rvvdk_vmdk::stream_map::StreamMap::read_from+0xb10>
   b71fc:	cmp    $0x1,%r13d
   b7200:	je     b764f <rvvdk_vmdk::stream_map::StreamMap::read_from+0xf4f>
   b7206:	movl   $0x0,0x30(%rsp)
   b720e:	jmp    b7228 <rvvdk_vmdk::stream_map::StreamMap::read_from+0xb28>
   b7210:	test   %r12d,%r12d
   b7213:	je     b719a <rvvdk_vmdk::stream_map::StreamMap::read_from+0xa9a>
   b7215:	mov    $0x1,%al
   b7217:	mov    %eax,0x30(%rsp)
   b721b:	mov    %r12d,%r13d
   b721e:	cmp    $0x1,%r12d
   b7222:	je     b764f <rvvdk_vmdk::stream_map::StreamMap::read_from+0xf4f>
   b7228:	mov    %r13d,%r13d
   b722b:	shl    $0x9,%r13
   b722f:	mov    $0x800,%edx
   b7234:	lea    0x5b0(%rsp),%rdi
   b723c:	mov    %r13,%rsi
   b723f:	call   *0x20fe5b(%rip)        # 2c70a0 <_DYNAMIC+0x610>
   b7245:	mov    0x5b0(%rsp),%rax
   b724d:	cmp    $0x10,%rax
   b7251:	jne    b766f <rvvdk_vmdk::stream_map::StreamMap::read_from+0xf6f>
   b7257:	add    $0xfffffffffffffe00,%r13
   b725e:	cmp    0x8(%rsp),%r13
   b7263:	jb     b76bc <rvvdk_vmdk::stream_map::StreamMap::read_from+0xfbc>
   b7269:	mov    0x5b8(%rsp),%rax
   b7271:	mov    %rax,0x8(%rsp)
   b7276:	cmp    0x28(%rsp),%rax
   b727b:	ja     b76bc <rvvdk_vmdk::stream_map::StreamMap::read_from+0xfbc>
   b7281:	mov    $0xa00,%ecx
   b7286:	lea    0x5b0(%rsp),%rdi
   b728e:	lea    0xe8(%rsp),%rsi
   b7296:	mov    %r13,%rdx
   b7299:	call   b8670 <rvvdk_vmdk::stream_map::StreamMap::read_from::{{closure}}>
   b729e:	cmpl   $0x10,0x5b0(%rsp)
   b72a6:	jne    b74e9 <rvvdk_vmdk::stream_map::StreamMap::read_from+0xde9>
   b72ac:	cmpb   $0x0,0x30(%rsp)
   b72b1:	jne    b719a <rvvdk_vmdk::stream_map::StreamMap::read_from+0xa9a>
   b72b7:	test   %r12d,%r12d
   b72ba:	je     b719a <rvvdk_vmdk::stream_map::StreamMap::read_from+0xa9a>
   b72c0:	cmp    $0x1,%r12d
   b72c4:	je     b764f <rvvdk_vmdk::stream_map::StreamMap::read_from+0xf4f>
   b72ca:	mov    %r12d,%r12d
   b72cd:	shl    $0x9,%r12
   b72d1:	mov    $0x800,%edx
   b72d6:	lea    0x5b0(%rsp),%rdi
   b72de:	mov    %r12,%rsi
   b72e1:	call   *0x20fdb9(%rip)        # 2c70a0 <_DYNAMIC+0x610>
   b72e7:	mov    0x5b0(%rsp),%rax
   b72ef:	cmp    $0x10,%rax
   b72f3:	jne    b766f <rvvdk_vmdk::stream_map::StreamMap::read_from+0xf6f>
   b72f9:	add    $0xfffffffffffffe00,%r12
   b7300:	cmp    0x8(%rsp),%r12
   b7305:	jb     b76bc <rvvdk_vmdk::stream_map::StreamMap::read_from+0xfbc>
   b730b:	mov    0x5b8(%rsp),%rax
   b7313:	mov    %rax,0x8(%rsp)
   b7318:	cmp    0x28(%rsp),%rax
   b731d:	ja     b76bc <rvvdk_vmdk::stream_map::StreamMap::read_from+0xfbc>
   b7323:	mov    $0xa00,%ecx
   b7328:	lea    0x5b0(%rsp),%rdi
   b7330:	lea    0xe8(%rsp),%rsi
   b7338:	mov    %r12,%rdx
   b733b:	call   b8670 <rvvdk_vmdk::stream_map::StreamMap::read_from::{{closure}}>
   b7340:	cmpl   $0x10,0x5b0(%rsp)
   b7348:	je     b719a <rvvdk_vmdk::stream_map::StreamMap::read_from+0xa9a>
   b734e:	jmp    b74e9 <rvvdk_vmdk::stream_map::StreamMap::read_from+0xde9>
   b7353:	lea    0x202bde(%rip),%rcx        # 2b9f38 <anon.c1dd2dbd4cf59c2854d815f601d2b4b9.44.llvm.4207380642161847320+0x78>
   b735a:	mov    %r13,0x50(%rsp)
   b735f:	jmp    b75c6 <rvvdk_vmdk::stream_map::StreamMap::read_from+0xec6>
   b7364:	xor    %r12d,%r12d
   b7367:	jmp    b7377 <rvvdk_vmdk::stream_map::StreamMap::read_from+0xc77>
   b7369:	inc    %r12
   b736c:	cmp    %r12,0x78(%rsp)
   b7371:	je     b7537 <rvvdk_vmdk::stream_map::StreamMap::read_from+0xe37>
   b7377:	mov    0x18(%rsp),%rdi
   b737c:	mov    (%rsp),%rsi
   b7380:	mov    %r12,%rdx
   b7383:	call   *0x20fd0f(%rip)        # 2c7098 <_DYNAMIC+0x608>
   b7389:	mov    %eax,%r13d
   b738c:	xor    %r14d,%r14d
   b738f:	cmpl   $0x1,0x10(%rsp)
   b7394:	jne    b73c0 <rvvdk_vmdk::stream_map::StreamMap::read_from+0xcc0>
   b7396:	mov    0x40(%rsp),%rdi
   b739b:	mov    0x50(%rsp),%rsi
   b73a0:	mov    %r12,%rdx
   b73a3:	call   *0x20fcef(%rip)        # 2c7098 <_DYNAMIC+0x608>
   b73a9:	mov    %eax,%r14d
   b73ac:	test   %r13d,%r13d
   b73af:	sete   %al
   b73b2:	test   %r14d,%r14d
   b73b5:	sete   %cl
   b73b8:	xor    %al,%cl
   b73ba:	jne    b7691 <rvvdk_vmdk::stream_map::StreamMap::read_from+0xf91>
   b73c0:	test   %r13d,%r13d
   b73c3:	je     b73d9 <rvvdk_vmdk::stream_map::StreamMap::read_from+0xcd9>
   b73c5:	cmp    $0x1,%r13d
   b73c9:	je     b764f <rvvdk_vmdk::stream_map::StreamMap::read_from+0xf4f>
   b73cf:	movl   $0x0,0x30(%rsp)
   b73d7:	jmp    b73f1 <rvvdk_vmdk::stream_map::StreamMap::read_from+0xcf1>
   b73d9:	test   %r14d,%r14d
   b73dc:	je     b7369 <rvvdk_vmdk::stream_map::StreamMap::read_from+0xc69>
   b73de:	mov    $0x1,%al
   b73e0:	mov    %eax,0x30(%rsp)
   b73e4:	mov    %r14d,%r13d
   b73e7:	cmp    $0x1,%r14d
   b73eb:	je     b764f <rvvdk_vmdk::stream_map::StreamMap::read_from+0xf4f>
   b73f1:	mov    %r13d,%r13d
   b73f4:	shl    $0x9,%r13
   b73f8:	mov    $0x800,%edx
   b73fd:	lea    0x5b0(%rsp),%rdi
   b7405:	mov    %r13,%rsi
   b7408:	call   *0x20fc92(%rip)        # 2c70a0 <_DYNAMIC+0x610>
   b740e:	mov    0x5b0(%rsp),%rax
   b7416:	cmp    $0x10,%rax
   b741a:	jne    b766f <rvvdk_vmdk::stream_map::StreamMap::read_from+0xf6f>
   b7420:	mov    0x8(%rsp),%rax
   b7425:	cmp    %rax,0x5b8(%rsp)
   b742d:	ja     b76dc <rvvdk_vmdk::stream_map::StreamMap::read_from+0xfdc>
   b7433:	mov    $0x800,%ecx
   b7438:	lea    0x5b0(%rsp),%rdi
   b7440:	lea    0xe8(%rsp),%rsi
   b7448:	mov    %r13,%rdx
   b744b:	call   b8670 <rvvdk_vmdk::stream_map::StreamMap::read_from::{{closure}}>
   b7450:	cmpl   $0x10,0x5b0(%rsp)
   b7458:	jne    b74e9 <rvvdk_vmdk::stream_map::StreamMap::read_from+0xde9>
   b745e:	cmpb   $0x0,0x30(%rsp)
   b7463:	jne    b7369 <rvvdk_vmdk::stream_map::StreamMap::read_from+0xc69>
   b7469:	test   %r14d,%r14d
   b746c:	je     b7369 <rvvdk_vmdk::stream_map::StreamMap::read_from+0xc69>
   b7472:	cmp    $0x1,%r14d
   b7476:	je     b764f <rvvdk_vmdk::stream_map::StreamMap::read_from+0xf4f>
   b747c:	mov    %r14d,%r13d
   b747f:	shl    $0x9,%r13
   b7483:	mov    $0x800,%edx
   b7488:	lea    0x5b0(%rsp),%rdi
   b7490:	mov    %r13,%rsi
   b7493:	call   *0x20fc07(%rip)        # 2c70a0 <_DYNAMIC+0x610>
   b7499:	mov    0x5b0(%rsp),%rax
   b74a1:	cmp    $0x10,%rax
   b74a5:	jne    b766f <rvvdk_vmdk::stream_map::StreamMap::read_from+0xf6f>
   b74ab:	mov    0x8(%rsp),%rax
   b74b0:	cmp    %rax,0x5b8(%rsp)
   b74b8:	ja     b76dc <rvvdk_vmdk::stream_map::StreamMap::read_from+0xfdc>
   b74be:	mov    $0x800,%ecx
   b74c3:	lea    0x5b0(%rsp),%rdi
   b74cb:	lea    0xe8(%rsp),%rsi
   b74d3:	mov    %r13,%rdx
   b74d6:	call   b8670 <rvvdk_vmdk::stream_map::StreamMap::read_from::{{closure}}>
   b74db:	cmpl   $0x10,0x5b0(%rsp)
   b74e3:	je     b7369 <rvvdk_vmdk::stream_map::StreamMap::read_from+0xc69>
   b74e9:	movups 0x5b0(%rsp),%xmm0
   b74f1:	movups 0x5c0(%rsp),%xmm1
   b74f9:	movups %xmm1,0x18(%r15)
   b74fe:	movups %xmm0,0x8(%r15)
   b7503:	movq   $0x3,(%r15)
   b750a:	mov    0xb0(%rsp),%rsi
   b7512:	test   %rsi,%rsi
   b7515:	je     b6fbf <rvvdk_vmdk::stream_map::StreamMap::read_from+0x8bf>
   b751b:	mov    0xb8(%rsp),%rdi
   b7523:	shl    $0x4,%rsi
   b7527:	mov    $0x8,%edx
   b752c:	call   *0x20f7ae(%rip)        # 2c6ce0 <_DYNAMIC+0x250>
   b7532:	jmp    b6fbf <rvvdk_vmdk::stream_map::StreamMap::read_from+0x8bf>
   b7537:	mov    0xd8(%rsp),%rsi
   b753f:	mov    0xc0(%rsp),%rdx
   b7547:	cmp    %rdx,%rsi
   b754a:	ja     b78e5 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x11e5>
   b7550:	mov    0xb8(%rsp),%rdi
   b7558:	call   b8980 <core::slice::<impl [T]>::sort_unstable_by_key>
   b755d:	mov    0xd8(%rsp),%rsi
   b7565:	mov    0xc0(%rsp),%rdx
   b756d:	cmp    %rdx,%rsi
   b7570:	ja     b78ee <rvvdk_vmdk::stream_map::StreamMap::read_from+0x11ee>
   b7576:	mov    0xb8(%rsp),%rax
   b757e:	mov    %rax,0x5b0(%rsp)
   b7586:	mov    %rsi,0x5b8(%rsp)
   b758e:	movq   $0x2,0x5c0(%rsp)
   b759a:	lea    0x5b0(%rsp),%rdi
   b75a2:	call   b8910 <core::iter::traits::iterator::Iterator::try_fold>
   b75a7:	test   %al,%al
   b75a9:	je     b75d9 <rvvdk_vmdk::stream_map::StreamMap::read_from+0xed9>
   b75ab:	movq   $0xa,0x8(%r15)
   b75b3:	lea    -0x96e2d(%rip),%rax        # 2078d <anon.c1dd2dbd4cf59c2854d815f601d2b4b9.42.llvm.4207380642161847320+0x1a4>
   b75ba:	jmp    b765e <rvvdk_vmdk::stream_map::StreamMap::read_from+0xf5e>
   b75bf:	lea    0x20295a(%rip),%rcx        # 2b9f20 <anon.c1dd2dbd4cf59c2854d815f601d2b4b9.44.llvm.4207380642161847320+0x60>
   b75c6:	mov    0x50(%rsp),%rdx
   b75cb:	mov    %rdx,%rsi
   b75ce:	call   *0x20f93c(%rip)        # 2c6f10 <_DYNAMIC+0x480>
   b75d4:	jmp    b85ac <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1eac>
   b75d9:	lea    0x5b0(%rsp),%rdi
   b75e1:	mov    $0x200,%edx
   b75e6:	mov    0x38(%rsp),%rsi
   b75eb:	call   *0x20fab7(%rip)        # 2c70a8 <_DYNAMIC+0x618>
   b75f1:	mov    0x5b0(%rsp),%rax
   b75f9:	mov    0x5b8(%rsp),%rsi
   b7601:	cmp    $0x10,%rax
   b7605:	jne    b77d6 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x10d6>
   b760b:	lea    0xdb0(%rsp),%rdi
   b7613:	mov    $0x2,%edx
   b7618:	call   *0x20fa8a(%rip)        # 2c70a8 <_DYNAMIC+0x618>
   b761e:	mov    0xdb0(%rsp),%rax
   b7626:	mov    0xdb8(%rsp),%r13
   b762e:	cmp    $0x10,%rax
   b7632:	jne    b76a2 <rvvdk_vmdk::stream_map::StreamMap::read_from+0xfa2>
   b7634:	cmp    0x68(%rbp),%r13
   b7638:	jbe    b76fc <rvvdk_vmdk::stream_map::StreamMap::read_from+0xffc>
   b763e:	movq   $0xb,0x8(%r15)
   b7646:	lea    -0x96fce(%rip),%rax        # 2067f <anon.c1dd2dbd4cf59c2854d815f601d2b4b9.42.llvm.4207380642161847320+0x96>
   b764d:	jmp    b765e <rvvdk_vmdk::stream_map::StreamMap::read_from+0xf5e>
   b764f:	movq   $0xa,0x8(%r15)
   b7657:	lea    -0x96ea0(%rip),%rax        # 207be <anon.c1dd2dbd4cf59c2854d815f601d2b4b9.42.llvm.4207380642161847320+0x1d5>
   b765e:	mov    %rax,0x10(%r15)
   b7662:	movq   $0x16,0x18(%r15)
   b766a:	jmp    b7503 <rvvdk_vmdk::stream_map::StreamMap::read_from+0xe03>
   b766f:	mov    0x5b8(%rsp),%rcx
   b7677:	movups 0x5c0(%rsp),%xmm0
   b767f:	movups %xmm0,0x18(%r15)
   b7684:	mov    %rax,0x8(%r15)
   b7688:	mov    %rcx,0x10(%r15)
   b768c:	jmp    b7503 <rvvdk_vmdk::stream_map::StreamMap::read_from+0xe03>
   b7691:	movq   $0xa,0x8(%r15)
   b7699:	lea    -0x96efd(%rip),%rax        # 207a3 <anon.c1dd2dbd4cf59c2854d815f601d2b4b9.42.llvm.4207380642161847320+0x1ba>
   b76a0:	jmp    b76cb <rvvdk_vmdk::stream_map::StreamMap::read_from+0xfcb>
   b76a2:	movups 0xdc0(%rsp),%xmm0
   b76aa:	movups %xmm0,0x18(%r15)
   b76af:	mov    %rax,0x8(%r15)
   b76b3:	mov    %r13,0x10(%r15)
   b76b7:	jmp    b7503 <rvvdk_vmdk::stream_map::StreamMap::read_from+0xe03>
   b76bc:	movq   $0xa,0x8(%r15)
   b76c4:	lea    -0x96ee3(%rip),%rax        # 207e8 <anon.c1dd2dbd4cf59c2854d815f601d2b4b9.42.llvm.4207380642161847320+0x1ff>
   b76cb:	mov    %rax,0x10(%r15)
   b76cf:	movq   $0x1b,0x18(%r15)
   b76d7:	jmp    b7503 <rvvdk_vmdk::stream_map::StreamMap::read_from+0xe03>
   b76dc:	movq   $0xa,0x8(%r15)
   b76e4:	lea    -0x96f17(%rip),%rax        # 207d4 <anon.c1dd2dbd4cf59c2854d815f601d2b4b9.42.llvm.4207380642161847320+0x1eb>
   b76eb:	mov    %rax,0x10(%r15)
   b76ef:	movq   $0x14,0x18(%r15)
   b76f7:	jmp    b7503 <rvvdk_vmdk::stream_map::StreamMap::read_from+0xe03>
   b76fc:	mov    0x110(%rsp),%r12
   b7704:	lea    0x5b0(%rsp),%rdi
   b770c:	mov    $0x800,%edx
   b7711:	mov    0x38(%rsp),%rsi
   b7716:	call   *0x20f98c(%rip)        # 2c70a8 <_DYNAMIC+0x618>
   b771c:	mov    0x5b0(%rsp),%rax
   b7724:	mov    0x5b8(%rsp),%rsi
   b772c:	cmp    $0x10,%rax
   b7730:	jne    b77d6 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x10d6>
   b7736:	lea    0xdb0(%rsp),%rdi
   b773e:	mov    0x20(%rsp),%rdx
   b7743:	call   *0x20f95f(%rip)        # 2c70a8 <_DYNAMIC+0x618>
   b7749:	mov    0xdb0(%rsp),%rax
   b7751:	mov    0xdb8(%rsp),%rsi
   b7759:	cmp    $0x10,%rax
   b775d:	jne    b77f0 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x10f0>
   b7763:	lea    0x350(%rsp),%rdi
   b776b:	mov    $0x2,%edx
   b7770:	call   *0x20f932(%rip)        # 2c70a8 <_DYNAMIC+0x618>
   b7776:	mov    0x350(%rsp),%rax
   b777e:	mov    0x358(%rsp),%r14
   b7786:	cmp    $0x10,%rax
   b778a:	jne    b77fa <rvvdk_vmdk::stream_map::StreamMap::read_from+0x10fa>
   b778c:	cmpl   $0x2,0x98(%rsp)
   b7794:	jne    b7814 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1114>
   b7796:	lea    0x5b0(%rsp),%rdi
   b779e:	mov    $0x10,%edx
   b77a3:	mov    0x38(%rsp),%rsi
   b77a8:	call   *0x20f8fa(%rip)        # 2c70a8 <_DYNAMIC+0x618>
   b77ae:	mov    0x5b0(%rsp),%rax
   b77b6:	mov    0x5b8(%rsp),%rdx
   b77be:	cmp    $0x10,%rax
   b77c2:	je     b7816 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1116>
   b77c4:	movups 0x5c0(%rsp),%xmm0
   b77cc:	movups %xmm0,0x18(%r15)
   b77d1:	mov    %rdx,%r14
   b77d4:	jmp    b7807 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1107>
   b77d6:	movups 0x5c0(%rsp),%xmm0
   b77de:	movups %xmm0,0x18(%r15)
   b77e3:	mov    %rax,0x8(%r15)
   b77e7:	mov    %rsi,0x10(%r15)
   b77eb:	jmp    b7503 <rvvdk_vmdk::stream_map::StreamMap::read_from+0xe03>
   b77f0:	movups 0xdc0(%rsp),%xmm0
   b77f8:	jmp    b77de <rvvdk_vmdk::stream_map::StreamMap::read_from+0x10de>
   b77fa:	movups 0x360(%rsp),%xmm0
   b7802:	movups %xmm0,0x18(%r15)
   b7807:	mov    %rax,0x8(%r15)
   b780b:	mov    %r14,0x10(%r15)
   b780f:	jmp    b7503 <rvvdk_vmdk::stream_map::StreamMap::read_from+0xe03>
   b7814:	xor    %edx,%edx
   b7816:	lea    0x120(%rsp),%rdi
   b781e:	mov    %r14,%rsi
   b7821:	call   *0x20f879(%rip)        # 2c70a0 <_DYNAMIC+0x610>
   b7827:	mov    0x120(%rsp),%rax
   b782f:	mov    0x128(%rsp),%rdx
   b7837:	cmp    $0x10,%rax
   b783b:	jne    b78c1 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x11c1>
   b7841:	lea    0x1c0(%rsp),%rdi
   b7849:	mov    $0x200,%esi
   b784e:	call   *0x20f84c(%rip)        # 2c70a0 <_DYNAMIC+0x610>
   b7854:	mov    0x1c0(%rsp),%rax
   b785c:	mov    0x1c8(%rsp),%rdx
   b7864:	cmp    $0x10,%rax
   b7868:	jne    b78cb <rvvdk_vmdk::stream_map::StreamMap::read_from+0x11cb>
   b786a:	lea    0x2d0(%rsp),%rdi
   b7872:	mov    %r12,%rsi
   b7875:	call   *0x20f825(%rip)        # 2c70a0 <_DYNAMIC+0x610>
   b787b:	mov    0x2d0(%rsp),%rax
   b7883:	mov    0x2d8(%rsp),%rcx
   b788b:	mov    %rcx,0x8(%rsp)
   b7890:	cmp    $0x10,%rax
   b7894:	jne    b7902 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1202>
   b7896:	cmp    %rbx,0x8(%rsp)
   b789b:	jbe    b7921 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1221>
   b78a1:	movq   $0xb,0x8(%r15)
   b78a9:	lea    -0x9724d(%rip),%rax        # 20663 <anon.c1dd2dbd4cf59c2854d815f601d2b4b9.42.llvm.4207380642161847320+0x7a>
   b78b0:	mov    %rax,0x10(%r15)
   b78b4:	movq   $0xe,0x18(%r15)
   b78bc:	jmp    b7503 <rvvdk_vmdk::stream_map::StreamMap::read_from+0xe03>
   b78c1:	movups 0x130(%rsp),%xmm0
   b78c9:	jmp    b78d3 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x11d3>
   b78cb:	movups 0x1d0(%rsp),%xmm0
   b78d3:	movups %xmm0,0x18(%r15)
   b78d8:	mov    %rax,0x8(%r15)
   b78dc:	mov    %rdx,0x10(%r15)
   b78e0:	jmp    b7503 <rvvdk_vmdk::stream_map::StreamMap::read_from+0xe03>
   b78e5:	lea    0x20261c(%rip),%rcx        # 2b9f08 <anon.c1dd2dbd4cf59c2854d815f601d2b4b9.44.llvm.4207380642161847320+0x48>
   b78ec:	jmp    b78f5 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x11f5>
   b78ee:	lea    0x2025fb(%rip),%rcx        # 2b9ef0 <anon.c1dd2dbd4cf59c2854d815f601d2b4b9.44.llvm.4207380642161847320+0x30>
   b78f5:	xor    %edi,%edi
   b78f7:	call   *0x20f613(%rip)        # 2c6f10 <_DYNAMIC+0x480>
   b78fd:	jmp    b85ac <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1eac>
   b7902:	movups 0x2e0(%rsp),%xmm0
   b790a:	movups %xmm0,0x18(%r15)
   b790f:	mov    %rax,0x8(%r15)
   b7913:	mov    0x8(%rsp),%rax
   b7918:	mov    %rax,0x10(%r15)
   b791c:	jmp    b7503 <rvvdk_vmdk::stream_map::StreamMap::read_from+0xe03>
   b7921:	movq   $0x0,0x30(%rsp)
   b792a:	mov    0x20f5e7(%rip),%r14        # 2c6f18 <memset@GLIBC_2.2.5>
   b7931:	mov    $0x800,%edx
   b7936:	lea    0xdb0(%rsp),%rdi
   b793e:	xor    %esi,%esi
   b7940:	call   *%r14
   b7943:	mov    $0x800,%edx
   b7948:	lea    0x5b0(%rsp),%rdi
   b7950:	xor    %esi,%esi
   b7952:	call   *%r14
   b7955:	mov    0x298(%rsp),%rax
   b795d:	mov    %rax,0x28(%rsp)
   b7962:	mov    0x70(%rbp),%rax
   b7966:	mov    %rax,0x48(%rsp)
   b796b:	movl   $0x0,0x38(%rsp)
   b7973:	xor    %r14d,%r14d
   b7976:	mov    %r14,%r12
   b7979:	shl    $0x9,%r12
   b797d:	add    $0xfffffffffffffe00,%r12
   b7984:	cmp    0x78(%rsp),%r14
   b7989:	jae    b7b17 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1417>
   b798f:	mov    0x18(%rsp),%rdi
   b7994:	mov    (%rsp),%rsi
   b7998:	mov    %r14,%rdx
   b799b:	call   *0x20f6f7(%rip)        # 2c7098 <_DYNAMIC+0x608>
   b79a1:	inc    %r14
   b79a4:	add    $0x200,%r12
   b79ab:	test   %eax,%eax
   b79ad:	je     b7984 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1284>
   b79af:	lea    -0x1(%r14),%rax
   b79b3:	sub    $0x8,%rsp
   b79b7:	lea    0x358(%rsp),%rdi
   b79bf:	lea    0x108(%rsp),%rsi
   b79c7:	mov    0x20(%rsp),%rdx
   b79cc:	mov    0x8(%rsp),%rcx
   b79d1:	mov    0x48(%rsp),%r8
   b79d6:	mov    0x58(%rsp),%r9
   b79db:	lea    0x5b8(%rsp),%r10
   b79e3:	push   %r10
   b79e5:	lea    0xdc0(%rsp),%r10
   b79ed:	push   %r10
   b79ef:	push   %rax
   b79f0:	call   b62c0 <rvvdk_vmdk::stream_map::read_table>
   b79f5:	add    $0x20,%rsp
   b79f9:	cmpl   $0x10,0x350(%rsp)
   b7a01:	jne    b80aa <rvvdk_vmdk::stream_map::StreamMap::read_from+0x19aa>
   b7a07:	movq   $0x0,0x10(%rsp)
   b7a10:	cmpq   $0x1ff,0x10(%rsp)
   b7a19:	jbe    b7a45 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1345>
   b7a1b:	jmp    b7976 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1276>
   b7a20:	cmpl   $0x0,0x20(%rsp)
   b7a25:	jne    b7bd9 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x14d9>
   b7a2b:	mov    0x10(%rsp),%rcx
   b7a30:	inc    %rcx
   b7a33:	mov    %rcx,0x10(%rsp)
   b7a38:	cmp    $0x200,%rcx
   b7a3f:	je     b7976 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1276>
   b7a45:	mov    $0x800,%esi
   b7a4a:	lea    0xdb0(%rsp),%rdi
   b7a52:	mov    0x10(%rsp),%rdx
   b7a57:	call   *0x20f63b(%rip)        # 2c7098 <_DYNAMIC+0x608>
   b7a5d:	mov    %eax,0x20(%rsp)
   b7a61:	mov    0x10(%rsp),%rax
   b7a66:	add    %r12,%rax
   b7a69:	cmp    0xd0(%rsp),%rax
   b7a71:	jae    b7a20 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1320>
   b7a73:	cmpl   $0x0,0x20(%rsp)
   b7a78:	je     b7a2b <rvvdk_vmdk::stream_map::StreamMap::read_from+0x132b>
   b7a7a:	cmpl   $0x1,0x20(%rsp)
   b7a7f:	je     b7bc5 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x14c5>
   b7a85:	mov    0x20(%rsp),%esi
   b7a89:	shl    $0x9,%rsi
   b7a8d:	cmp    0x28(%rsp),%rsi
   b7a92:	jb     b7bc5 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x14c5>
   b7a98:	mov    $0x200,%edx
   b7a9d:	lea    0x350(%rsp),%rdi
   b7aa5:	call   *0x20f5f5(%rip)        # 2c70a0 <_DYNAMIC+0x610>
   b7aab:	mov    0x350(%rsp),%rcx
   b7ab3:	mov    0x358(%rsp),%rax
   b7abb:	cmp    $0x10,%rcx
   b7abf:	jne    b80e6 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x19e6>
   b7ac5:	cmp    0x88(%rsp),%rax
   b7acd:	ja     b7bc5 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x14c5>
   b7ad3:	mov    0x20(%rsp),%eax
   b7ad7:	cmp    0x38(%rsp),%eax
   b7adb:	jbe    b810c <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1a0c>
   b7ae1:	incq   0x10(%rsp)
   b7ae6:	mov    0x30(%rsp),%rcx
   b7aeb:	inc    %rcx
   b7aee:	mov    0x20(%rsp),%eax
   b7af2:	mov    %eax,0x38(%rsp)
   b7af6:	mov    %rcx,0x30(%rsp)
   b7afb:	cmp    0x48(%rsp),%rcx
   b7b00:	jbe    b7a10 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1310>
   b7b06:	movq   $0xb,0x8(%r15)
   b7b0e:	lea    -0xa0535(%rip),%rax        # 175e0 <anon.e60e008f553044d7c15e66252433230a.30.llvm.15109012375404921823+0x80>
   b7b15:	jmp    b7b95 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1495>
   b7b17:	lea    0x350(%rsp),%rdi
   b7b1f:	mov    $0xc,%edx
   b7b24:	mov    0x30(%rsp),%rsi
   b7b29:	call   *0x20f579(%rip)        # 2c70a8 <_DYNAMIC+0x618>
   b7b2f:	mov    0x350(%rsp),%rax
   b7b37:	mov    0x358(%rsp),%rdx
   b7b3f:	cmp    $0x10,%rax
   b7b43:	jne    b8083 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1983>
   b7b49:	lea    0x120(%rsp),%rdi
   b7b51:	mov    0x58(%rsp),%rsi
   b7b56:	call   *0x20f544(%rip)        # 2c70a0 <_DYNAMIC+0x610>
   b7b5c:	mov    0x120(%rsp),%rax
   b7b64:	mov    0x128(%rsp),%rcx
   b7b6c:	mov    %rcx,0x10(%rsp)
   b7b71:	cmp    $0x10,%rax
   b7b75:	jne    b7ba6 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x14a6>
   b7b77:	mov    0x10(%rsp),%rax
   b7b7c:	cmp    0xa0(%rsp),%rax
   b7b84:	jbe    b7bf9 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x14f9>
   b7b86:	movq   $0xb,0x8(%r15)
   b7b8e:	lea    -0xa0895(%rip),%rax        # 17300 <anon.d625489d584c397ac22d75864c33158f.35.llvm.5936746164385555759+0x1a0>
   b7b95:	mov    %rax,0x10(%r15)
   b7b99:	movq   $0x10,0x18(%r15)
   b7ba1:	jmp    b7503 <rvvdk_vmdk::stream_map::StreamMap::read_from+0xe03>
   b7ba6:	movups 0x130(%rsp),%xmm0
   b7bae:	movups %xmm0,0x18(%r15)
   b7bb3:	mov    %rax,0x8(%r15)
   b7bb7:	mov    0x10(%rsp),%rax
   b7bbc:	mov    %rax,0x10(%r15)
   b7bc0:	jmp    b7503 <rvvdk_vmdk::stream_map::StreamMap::read_from+0xe03>
   b7bc5:	movq   $0xa,0x8(%r15)
   b7bcd:	lea    -0x9746d(%rip),%rax        # 20767 <anon.c1dd2dbd4cf59c2854d815f601d2b4b9.42.llvm.4207380642161847320+0x17e>
   b7bd4:	jmp    b76eb <rvvdk_vmdk::stream_map::StreamMap::read_from+0xfeb>
   b7bd9:	movq   $0xa,0x8(%r15)
   b7be1:	lea    -0x9746d(%rip),%rax        # 2077b <anon.c1dd2dbd4cf59c2854d815f601d2b4b9.42.llvm.4207380642161847320+0x192>
   b7be8:	mov    %rax,0x10(%r15)
   b7bec:	movq   $0x12,0x18(%r15)
   b7bf4:	jmp    b7503 <rvvdk_vmdk::stream_map::StreamMap::read_from+0xe03>
   b7bf9:	lea    0x350(%rsp),%rdi
   b7c01:	mov    $0xc,%edx
   b7c06:	mov    0x30(%rsp),%rsi
   b7c0b:	call   *0x20f497(%rip)        # 2c70a8 <_DYNAMIC+0x618>
   b7c11:	mov    0x350(%rsp),%rax
   b7c19:	mov    0x358(%rsp),%rdx
   b7c21:	cmp    $0x10,%rax
   b7c25:	jne    b8083 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1983>
   b7c2b:	lea    0x120(%rsp),%rdi
   b7c33:	mov    0x8(%rsp),%rsi
   b7c38:	call   *0x20f462(%rip)        # 2c70a0 <_DYNAMIC+0x610>
   b7c3e:	mov    0x120(%rsp),%rcx
   b7c46:	mov    0x128(%rsp),%rax
   b7c4e:	cmp    $0x10,%rcx
   b7c52:	jne    b8090 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1990>
   b7c58:	cmp    %rbx,%rax
   b7c5b:	ja     b78a1 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x11a1>
   b7c61:	movq   $0x0,0x120(%rsp)
   b7c6d:	movl   $0x0,0x128(%rsp)
   b7c78:	lea    0x350(%rsp),%rdi
   b7c80:	lea    0x120(%rsp),%rdx
   b7c88:	mov    0x30(%rsp),%rsi
   b7c8d:	call   b87e0 <rvvdk_vmdk::stream_map::allocated>
   b7c92:	mov    0x350(%rsp),%rax
   b7c9a:	mov    0x358(%rsp),%rcx
   b7ca2:	mov    %rcx,0x48(%rsp)
   b7ca7:	mov    0x360(%rsp),%rcx
   b7caf:	mov    %rcx,0x90(%rsp)
   b7cb7:	mov    0x368(%rsp),%rcx
   b7cbf:	mov    %rcx,0x70(%rsp)
   b7cc4:	cmp    $0x10,%rax
   b7cc8:	jne    b80bf <rvvdk_vmdk::stream_map::StreamMap::read_from+0x19bf>
   b7cce:	movq   $0x0,0x68(%rsp)
   b7cd7:	movq   $0x0,0x8(%rsp)
   b7ce0:	mov    0x8(%rsp),%rax
   b7ce5:	shl    $0x9,%rax
   b7ce9:	mov    %rax,0x20(%rsp)
   b7cee:	mov    0x8(%rsp),%r14
   b7cf3:	mov    0x20(%rsp),%rax
   b7cf8:	mov    %rax,0x58(%rsp)
   b7cfd:	cmp    0x78(%rsp),%r14
   b7d02:	jae    b80f0 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x19f0>
   b7d08:	mov    0x18(%rsp),%rdi
   b7d0d:	mov    (%rsp),%rsi
   b7d11:	mov    %r14,%rdx
   b7d14:	call   *0x20f37e(%rip)        # 2c7098 <_DYNAMIC+0x608>
   b7d1a:	mov    %eax,0xa0(%rsp)
   b7d21:	lea    0x1(%r14),%rax
   b7d25:	mov    %rax,0x8(%rsp)
   b7d2a:	mov    0x58(%rsp),%rax
   b7d2f:	add    $0x200,%rax
   b7d35:	mov    %rax,0x20(%rsp)
   b7d3a:	cmpl   $0x0,0xa0(%rsp)
   b7d42:	je     b7cee <rvvdk_vmdk::stream_map::StreamMap::read_from+0x15ee>
   b7d44:	sub    $0x8,%rsp
   b7d48:	lea    0x358(%rsp),%rdi
   b7d50:	lea    0x108(%rsp),%rsi
   b7d58:	mov    0x20(%rsp),%rdx
   b7d5d:	mov    0x8(%rsp),%rcx
   b7d62:	mov    0x48(%rsp),%r8
   b7d67:	mov    0x58(%rsp),%r9
   b7d6c:	lea    0x5b8(%rsp),%rax
   b7d74:	push   %rax
   b7d75:	lea    0xdc0(%rsp),%rax
   b7d7d:	push   %rax
   b7d7e:	push   %r14
   b7d80:	call   b62c0 <rvvdk_vmdk::stream_map::read_table>
   b7d85:	add    $0x20,%rsp
   b7d89:	cmpl   $0x10,0x350(%rsp)
   b7d91:	jne    b8287 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1b87>
   b7d97:	movq   $0x0,0xa8(%rsp)
   b7da3:	xor    %r14d,%r14d
   b7da6:	mov    0x58(%rsp),%rax
   b7dab:	mov    0xa8(%rsp),%rcx
   b7db3:	lea    (%rax,%rcx,1),%r12
   b7db7:	shl    $0x10,%r12
   b7dbb:	add    $0xffffffffffff0000,%r12
   b7dc2:	mov    $0x201,%ebx
   b7dc7:	cmp    $0x200,%r14
   b7dce:	jae    b7f46 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1846>
   b7dd4:	mov    $0x800,%esi
   b7dd9:	lea    0xdb0(%rsp),%rdi
   b7de1:	mov    %r14,%rdx
   b7de4:	call   *0x20f2ae(%rip)        # 2c7098 <_DYNAMIC+0x608>
   b7dea:	mov    %eax,0x38(%rsp)
   b7dee:	inc    %r14
   b7df1:	add    $0x10000,%r12
   b7df8:	dec    %rbx
   b7dfb:	cmpl   $0x0,0x38(%rsp)
   b7e00:	je     b7dc7 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x16c7>
   b7e02:	mov    0x68(%rsp),%rax
   b7e07:	cmp    0x70(%rsp),%rax
   b7e0c:	je     b849f <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1d9f>
   b7e12:	mov    0x20(%rsp),%rax
   b7e17:	mov    0xa8(%rsp),%rcx
   b7e1f:	add    %rcx,%rax
   b7e22:	sub    %rbx,%rax
   b7e25:	cmp    0xd0(%rsp),%rax
   b7e2d:	jae    b849f <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1d9f>
   b7e33:	mov    0x38(%rsp),%eax
   b7e37:	shl    $0x9,%rax
   b7e3b:	cmp    0x28(%rsp),%rax
   b7e40:	jne    b84bf <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1dbf>
   b7e46:	xorps  %xmm0,%xmm0
   b7e49:	movaps %xmm0,0x120(%rsp)
   b7e51:	mov    $0xc,%r8d
   b7e57:	lea    0x350(%rsp),%rdi
   b7e5f:	lea    0x100(%rsp),%rsi
   b7e67:	mov    0x28(%rsp),%rdx
   b7e6c:	lea    0x120(%rsp),%rcx
   b7e74:	call   b65a0 <rvvdk_vmdk::stream_map::Budget<S>::read>
   b7e79:	cmpl   $0x10,0x350(%rsp)
   b7e81:	jne    b8287 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1b87>
   b7e87:	mov    $0x10,%edx
   b7e8c:	lea    0x350(%rsp),%rdi
   b7e94:	lea    0x120(%rsp),%rsi
   b7e9c:	mov    0x28(%rsp),%rcx
   b7ea1:	lea    0x250(%rsp),%r8
   b7ea9:	mov    %rbp,%r9
   b7eac:	call   *0x20f10e(%rip)        # 2c6fc0 <_DYNAMIC+0x530>
   b7eb2:	mov    0x350(%rsp),%rcx
   b7eba:	mov    0x358(%rsp),%rax
   b7ec2:	mov    0x360(%rsp),%rdx
   b7eca:	mov    %rdx,0x28(%rsp)
   b7ecf:	cmp    $0x5,%rcx
   b7ed3:	je     b84d0 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1dd0>
   b7ed9:	test   %rcx,%rcx
   b7edc:	jne    b84ef <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1def>
   b7ee2:	cmp    %r12,%rax
   b7ee5:	jne    b8500 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1e00>
   b7eeb:	mov    0x68(%rsp),%rax
   b7ef0:	cmp    0x70(%rsp),%rax
   b7ef5:	jae    b8595 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1e95>
   b7efb:	mov    0x370(%rsp),%rax
   b7f03:	mov    0xa8(%rsp),%rdi
   b7f0b:	add    0x20(%rsp),%edi
   b7f0f:	sub    %ebx,%edi
   b7f11:	mov    0x68(%rsp),%rsi
   b7f16:	lea    (%rsi,%rsi,2),%rcx
   b7f1a:	mov    0x90(%rsp),%rdx
   b7f22:	mov    %edi,(%rdx,%rcx,4)
   b7f25:	mov    0x38(%rsp),%edi
   b7f29:	mov    %edi,0x4(%rdx,%rcx,4)
   b7f2d:	mov    %eax,0x8(%rdx,%rcx,4)
   b7f31:	inc    %rsi
   b7f34:	mov    %rsi,0x68(%rsp)
   b7f39:	mov    %r14,0xa8(%rsp)
   b7f41:	jmp    b7da6 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x16a6>
   b7f46:	cmpl   $0x2,0x98(%rsp)
   b7f4e:	jne    b7ce0 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x15e0>
   b7f54:	mov    0xa0(%rsp),%ebx
   b7f5b:	shl    $0x9,%rbx
   b7f5f:	lea    -0x200(%rbx),%rax
   b7f66:	cmp    %rax,0x28(%rsp)
   b7f6b:	jne    b8540 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1e40>
   b7f71:	xorps  %xmm0,%xmm0
   b7f74:	movaps %xmm0,0x2d0(%rsp)
   b7f7c:	mov    $0x10,%r8d
   b7f82:	lea    0x350(%rsp),%rdi
   b7f8a:	lea    0x100(%rsp),%rsi
   b7f92:	mov    0x28(%rsp),%rdx
   b7f97:	lea    0x2d0(%rsp),%rcx
   b7f9f:	call   b65a0 <rvvdk_vmdk::stream_map::Budget<S>::read>
   b7fa4:	cmpl   $0x10,0x350(%rsp)
   b7fac:	jne    b8287 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1b87>
   b7fb2:	mov    $0x10,%edx
   b7fb7:	lea    0x350(%rsp),%rdi
   b7fbf:	lea    0x2d0(%rsp),%rsi
   b7fc7:	mov    0x28(%rsp),%rcx
   b7fcc:	lea    0x250(%rsp),%r8
   b7fd4:	mov    %rbp,%r9
   b7fd7:	call   *0x20efe3(%rip)        # 2c6fc0 <_DYNAMIC+0x530>
   b7fdd:	mov    0x350(%rsp),%rax
   b7fe5:	lea    0x358(%rsp),%rcx
   b7fed:	movups (%rcx),%xmm0
   b7ff0:	movups 0x10(%rcx),%xmm1
   b7ff4:	movaps %xmm0,0x1c0(%rsp)
   b7ffc:	movaps %xmm1,0x1d0(%rsp)
   b8004:	cmp    $0x5,%rax
   b8008:	je     b8560 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1e60>
   b800e:	movaps 0x1c0(%rsp),%xmm0
   b8016:	movaps 0x1d0(%rsp),%xmm1
   b801e:	lea    0x128(%rsp),%rcx
   b8026:	movups %xmm1,0x10(%rcx)
   b802a:	movups %xmm0,(%rcx)
   b802d:	mov    %rax,0x120(%rsp)
   b8035:	mov    %rbx,0x358(%rsp)
   b803d:	movq   $0x800,0x360(%rsp)
   b8049:	movq   $0x1,0x350(%rsp)
   b8055:	lea    0x120(%rsp),%rdi
   b805d:	lea    0x350(%rsp),%rsi
   b8065:	call   b8a20 <<rvvdk_vmdk::stream::StreamMarker as core::cmp::PartialEq>::eq>
   b806a:	test   %al,%al
   b806c:	je     b8575 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1e75>
   b8072:	add    $0x800,%rbx
   b8079:	mov    %rbx,0x28(%rsp)
   b807e:	jmp    b7ce0 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x15e0>
   b8083:	movups 0x360(%rsp),%xmm0
   b808b:	jmp    b78d3 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x11d3>
   b8090:	movups 0x130(%rsp),%xmm0
   b8098:	movups %xmm0,0x18(%r15)
   b809d:	mov    %rcx,0x8(%r15)
   b80a1:	mov    %rax,0x10(%r15)
   b80a5:	jmp    b7503 <rvvdk_vmdk::stream_map::StreamMap::read_from+0xe03>
   b80aa:	movups 0x350(%rsp),%xmm0
   b80b2:	movups 0x360(%rsp),%xmm1
   b80ba:	jmp    b74f9 <rvvdk_vmdk::stream_map::StreamMap::read_from+0xdf9>
   b80bf:	mov    0x48(%rsp),%rcx
   b80c4:	mov    %rcx,0x10(%r15)
   b80c8:	mov    0x90(%rsp),%rcx
   b80d0:	mov    %rcx,0x18(%r15)
   b80d4:	mov    0x70(%rsp),%rcx
   b80d9:	mov    %rcx,0x20(%r15)
   b80dd:	mov    %rax,0x8(%r15)
   b80e1:	jmp    b7503 <rvvdk_vmdk::stream_map::StreamMap::read_from+0xe03>
   b80e6:	movups 0x360(%rsp),%xmm0
   b80ee:	jmp    b8098 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1998>
   b80f0:	cmpl   $0x2,0x98(%rsp)
   b80f8:	je     b812c <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1a2c>
   b80fa:	mov    0x88(%rsp),%rax
   b8102:	mov    %rax,0x80(%rsp)
   b810a:	jmp    b8138 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1a38>
   b810c:	movq   $0xa,0x8(%r15)
   b8114:	lea    -0x979cd(%rip),%rax        # 2074e <anon.c1dd2dbd4cf59c2854d815f601d2b4b9.42.llvm.4207380642161847320+0x165>
   b811b:	mov    %rax,0x10(%r15)
   b811f:	movq   $0x19,0x18(%r15)
   b8127:	jmp    b7503 <rvvdk_vmdk::stream_map::StreamMap::read_from+0xe03>
   b812c:	addq   $0xfffffffffffffe00,0x80(%rsp)
   b8138:	mov    0x28(%rsp),%rax
   b813d:	cmp    0x80(%rsp),%rax
   b8145:	jne    b8258 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1b58>
   b814b:	mov    0x68(%rsp),%rax
   b8150:	cmp    0x70(%rsp),%rax
   b8155:	jne    b8258 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1b58>
   b815b:	lea    0x350(%rsp),%r14
   b8163:	mov    $0x200,%edx
   b8168:	mov    %r14,%rdi
   b816b:	xor    %esi,%esi
   b816d:	call   *0x20eda5(%rip)        # 2c6f18 <memset@GLIBC_2.2.5>
   b8173:	lea    0x120(%rsp),%rdi
   b817b:	lea    0x100(%rsp),%rsi
   b8183:	mov    $0x200,%r8d
   b8189:	xor    %edx,%edx
   b818b:	mov    %r14,%rcx
   b818e:	call   b65a0 <rvvdk_vmdk::stream_map::Budget<S>::read>
   b8193:	cmpl   $0x10,0x120(%rsp)
   b819b:	jne    b8275 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1b75>
   b81a1:	mov    0x50(%rbp),%rax
   b81a5:	mov    %rax,0x320(%rsp)
   b81ad:	movups 0x40(%rbp),%xmm0
   b81b1:	movaps %xmm0,0x310(%rsp)
   b81b9:	movups 0x0(%rbp),%xmm0
   b81bd:	movups 0x10(%rbp),%xmm1
   b81c1:	movups 0x20(%rbp),%xmm2
   b81c5:	movups 0x30(%rbp),%xmm3
   b81c9:	movaps %xmm3,0x300(%rsp)
   b81d1:	movaps %xmm2,0x2f0(%rsp)
   b81d9:	movaps %xmm1,0x2e0(%rsp)
   b81e1:	movaps %xmm0,0x2d0(%rsp)
   b81e9:	mov    0x88(%rsp),%rcx
   b81f1:	lea    0x120(%rsp),%rdi
   b81f9:	lea    0x350(%rsp),%rsi
   b8201:	lea    0x2d0(%rsp),%r8
   b8209:	mov    $0x200,%edx
   b820e:	call   *0x20ed9c(%rip)        # 2c6fb0 <_DYNAMIC+0x520>
   b8214:	mov    0x120(%rsp),%rax
   b821c:	movups 0x128(%rsp),%xmm0
   b8224:	movaps %xmm0,0x220(%rsp)
   b822c:	movups 0x138(%rsp),%xmm0
   b8234:	movaps %xmm0,0x230(%rsp)
   b823c:	cmp    $0x3,%rax
   b8240:	jne    b82d9 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1bd9>
   b8246:	movaps 0x220(%rsp),%xmm0
   b824e:	movaps 0x230(%rsp),%xmm1
   b8256:	jmp    b8297 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1b97>
   b8258:	movq   $0xa,0x8(%r15)
   b8260:	lea    -0x97bae(%rip),%rax        # 206b9 <anon.c1dd2dbd4cf59c2854d815f601d2b4b9.42.llvm.4207380642161847320+0xd0>
   b8267:	mov    %rax,0x10(%r15)
   b826b:	movq   $0x26,0x18(%r15)
   b8273:	jmp    b82a1 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1ba1>
   b8275:	movups 0x120(%rsp),%xmm0
   b827d:	movups 0x130(%rsp),%xmm1
   b8285:	jmp    b8297 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1b97>
   b8287:	movups 0x350(%rsp),%xmm0
   b828f:	movups 0x360(%rsp),%xmm1
   b8297:	movups %xmm1,0x18(%r15)
   b829c:	movups %xmm0,0x8(%r15)
   b82a1:	movq   $0x3,(%r15)
   b82a8:	cmpq   $0x0,0x48(%rsp)
   b82ae:	je     b750a <rvvdk_vmdk::stream_map::StreamMap::read_from+0xe0a>
   b82b4:	mov    0x48(%rsp),%rax
   b82b9:	shl    $0x2,%rax
   b82bd:	lea    (%rax,%rax,2),%rsi
   b82c1:	mov    $0x4,%edx
   b82c6:	mov    0x90(%rsp),%rdi
   b82ce:	call   *0x20ea0c(%rip)        # 2c6ce0 <_DYNAMIC+0x250>
   b82d4:	jmp    b750a <rvvdk_vmdk::stream_map::StreamMap::read_from+0xe0a>
   b82d9:	mov    0x178(%rsp),%rcx
   b82e1:	mov    %rcx,0x218(%rsp)
   b82e9:	movups 0x148(%rsp),%xmm0
   b82f1:	movups 0x158(%rsp),%xmm1
   b82f9:	movups 0x168(%rsp),%xmm2
   b8301:	movups %xmm2,0x208(%rsp)
   b8309:	movups %xmm1,0x1f8(%rsp)
   b8311:	movups %xmm0,0x1e8(%rsp)
   b8319:	movaps 0x220(%rsp),%xmm0
   b8321:	movaps 0x230(%rsp),%xmm1
   b8329:	movups %xmm0,0x1c8(%rsp)
   b8331:	movups %xmm1,0x1d8(%rsp)
   b8339:	mov    %rax,0x1c0(%rsp)
   b8341:	lea    0x1c0(%rsp),%rdi
   b8349:	lea    0x250(%rsp),%rsi
   b8351:	call   b89d0 <<rvvdk_vmdk::stream::StreamHeader as core::cmp::PartialEq>::eq>
   b8356:	test   %al,%al
   b8358:	je     b848b <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1d8b>
   b835e:	mov    0x100(%rsp),%rax
   b8366:	mov    (%rax),%rcx
   b8369:	mov    0x10(%rcx),%rcx
   b836d:	mov    %rcx,0x8(%rax)
   b8371:	cmp    %rcx,0x88(%rsp)
   b8379:	jne    b8520 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1e20>
   b837f:	mov    0x110(%rsp),%rax
   b8387:	movaps 0x330(%rsp),%xmm0
   b838f:	movaps 0x340(%rsp),%xmm1
   b8397:	movups %xmm1,0x18(%r15)
   b839c:	movups %xmm0,0x8(%r15)
   b83a1:	mov    0x1b0(%rsp),%rcx
   b83a9:	mov    %rcx,0x58(%r15)
   b83ad:	movaps 0x180(%rsp),%xmm0
   b83b5:	movaps 0x190(%rsp),%xmm1
   b83bd:	movaps 0x1a0(%rsp),%xmm2
   b83c5:	movups %xmm2,0x48(%r15)
   b83ca:	movups %xmm1,0x38(%r15)
   b83cf:	movups %xmm0,0x28(%r15)
   b83d4:	mov    0x48(%rsp),%rcx
   b83d9:	mov    %rcx,0x60(%r15)
   b83dd:	mov    0x90(%rsp),%rcx
   b83e5:	mov    %rcx,0x68(%r15)
   b83e9:	mov    0x70(%rsp),%rcx
   b83ee:	mov    %rcx,0x70(%r15)
   b83f2:	mov    0x98(%rsp),%rcx
   b83fa:	mov    %rcx,(%r15)
   b83fd:	mov    %rax,0x78(%r15)
   b8401:	mov    %r13,0x80(%r15)
   b8408:	mov    0x30(%rsp),%rax
   b840d:	mov    %rax,0x88(%r15)
   b8414:	mov    0x10(%rsp),%rax
   b8419:	mov    %rax,0x90(%r15)
   b8420:	mov    0xe4(%rsp),%eax
   b8427:	mov    %eax,0x98(%r15)
   b842e:	mov    0xb0(%rsp),%rsi
   b8436:	test   %rsi,%rsi
   b8439:	je     b8452 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1d52>
   b843b:	mov    0xb8(%rsp),%rdi
   b8443:	shl    $0x4,%rsi
   b8447:	mov    $0x8,%edx
   b844c:	call   *0x20e88e(%rip)        # 2c6ce0 <_DYNAMIC+0x250>
   b8452:	cmpq   $0x0,0x60(%rsp)
   b8458:	je     b846f <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1d6f>
   b845a:	mov    $0x1,%edx
   b845f:	mov    0x40(%rsp),%rdi
   b8464:	mov    0x60(%rsp),%rsi
   b8469:	call   *0x20e871(%rip)        # 2c6ce0 <_DYNAMIC+0x250>
   b846f:	mov    (%rsp),%rsi
   b8473:	test   %rsi,%rsi
   b8476:	mov    0x18(%rsp),%rdi
   b847b:	je     b6f65 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x865>
   b8481:	mov    $0x1,%edx
   b8486:	jmp    b6f5f <rvvdk_vmdk::stream_map::StreamMap::read_from+0x85f>
   b848b:	movq   $0xa,0x8(%r15)
   b8493:	lea    -0x97df3(%rip),%rax        # 206a7 <anon.c1dd2dbd4cf59c2854d815f601d2b4b9.42.llvm.4207380642161847320+0xbe>
   b849a:	jmp    b852f <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1e2f>
   b849f:	movq   $0xa,0x8(%r15)
   b84a7:	lea    -0x97d73(%rip),%rax        # 2073b <anon.c1dd2dbd4cf59c2854d815f601d2b4b9.42.llvm.4207380642161847320+0x152>
   b84ae:	mov    %rax,0x10(%r15)
   b84b2:	movq   $0x13,0x18(%r15)
   b84ba:	jmp    b82a1 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1ba1>
   b84bf:	movq   $0xa,0x8(%r15)
   b84c7:	lea    -0x97da8(%rip),%rax        # 20726 <anon.c1dd2dbd4cf59c2854d815f601d2b4b9.42.llvm.4207380642161847320+0x13d>
   b84ce:	jmp    b854f <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1e4f>
   b84d0:	movups 0x368(%rsp),%xmm0
   b84d8:	mov    %rax,0x8(%r15)
   b84dc:	mov    0x28(%rsp),%rax
   b84e1:	mov    %rax,0x10(%r15)
   b84e5:	movups %xmm0,0x18(%r15)
   b84ea:	jmp    b82a1 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1ba1>
   b84ef:	movq   $0xa,0x8(%r15)
   b84f7:	lea    -0x97ded(%rip),%rax        # 20711 <anon.c1dd2dbd4cf59c2854d815f601d2b4b9.42.llvm.4207380642161847320+0x128>
   b84fe:	jmp    b854f <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1e4f>
   b8500:	movq   $0xa,0x8(%r15)
   b8508:	lea    -0x97e0f(%rip),%rax        # 20700 <anon.c1dd2dbd4cf59c2854d815f601d2b4b9.42.llvm.4207380642161847320+0x117>
   b850f:	mov    %rax,0x10(%r15)
   b8513:	movq   $0x11,0x18(%r15)
   b851b:	jmp    b82a1 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1ba1>
   b8520:	movq   $0xa,0x8(%r15)
   b8528:	lea    -0x97e9a(%rip),%rax        # 20695 <anon.c1dd2dbd4cf59c2854d815f601d2b4b9.42.llvm.4207380642161847320+0xac>
   b852f:	mov    %rax,0x10(%r15)
   b8533:	movq   $0x12,0x18(%r15)
   b853b:	jmp    b82a1 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1ba1>
   b8540:	movq   $0xa,0x8(%r15)
   b8548:	lea    -0x97e64(%rip),%rax        # 206eb <anon.c1dd2dbd4cf59c2854d815f601d2b4b9.42.llvm.4207380642161847320+0x102>
   b854f:	mov    %rax,0x10(%r15)
   b8553:	movq   $0x15,0x18(%r15)
   b855b:	jmp    b82a1 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1ba1>
   b8560:	movaps 0x1c0(%rsp),%xmm0
   b8568:	movaps 0x1d0(%rsp),%xmm1
   b8570:	jmp    b8297 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1b97>
   b8575:	movq   $0xa,0x8(%r15)
   b857d:	lea    -0x97ea5(%rip),%rax        # 206df <anon.c1dd2dbd4cf59c2854d815f601d2b4b9.42.llvm.4207380642161847320+0xf6>
   b8584:	mov    %rax,0x10(%r15)
   b8588:	movq   $0xc,0x18(%r15)
   b8590:	jmp    b82a1 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1ba1>
   b8595:	lea    0x20193c(%rip),%rdx        # 2b9ed8 <anon.c1dd2dbd4cf59c2854d815f601d2b4b9.44.llvm.4207380642161847320+0x18>
   b859c:	mov    0x68(%rsp),%rdi
   b85a1:	mov    0x70(%rsp),%rsi
   b85a6:	call   *0x20e7cc(%rip)        # 2c6d78 <_DYNAMIC+0x2e8>
   b85ac:	ud2
   b85ae:	jmp    b85b6 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1eb6>
   b85b0:	jmp    b85b6 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1eb6>
   b85b2:	jmp    b85b6 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1eb6>
   b85b4:	jmp    b85b6 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1eb6>
   b85b6:	mov    %rax,%rbx
   b85b9:	cmpq   $0x0,0x48(%rsp)
   b85bf:	je     b85fa <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1efa>
   b85c1:	mov    0x48(%rsp),%rax
   b85c6:	shl    $0x2,%rax
   b85ca:	lea    (%rax,%rax,2),%rsi
   b85ce:	mov    $0x4,%edx
   b85d3:	mov    0x90(%rsp),%rdi
   b85db:	call   *0x20e6ff(%rip)        # 2c6ce0 <_DYNAMIC+0x250>
   b85e1:	jmp    b85fa <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1efa>
   b85e3:	jmp    b85f7 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1ef7>
   b85e5:	jmp    b85f7 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1ef7>
   b85e7:	jmp    b85f7 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1ef7>
   b85e9:	jmp    b85f7 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1ef7>
   b85eb:	jmp    b85f7 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1ef7>
   b85ed:	jmp    b85f7 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1ef7>
   b85ef:	jmp    b85f7 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1ef7>
   b85f1:	jmp    b85f7 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1ef7>
   b85f3:	jmp    b85f7 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1ef7>
   b85f5:	jmp    b85f7 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1ef7>
   b85f7:	mov    %rax,%rbx
   b85fa:	mov    0xb0(%rsp),%rsi
   b8602:	test   %rsi,%rsi
   b8605:	je     b8623 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1f23>
   b8607:	mov    0xb8(%rsp),%rdi
   b860f:	shl    $0x4,%rsi
   b8613:	mov    $0x8,%edx
   b8618:	call   *0x20e6c2(%rip)        # 2c6ce0 <_DYNAMIC+0x250>
   b861e:	jmp    b8623 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1f23>
   b8620:	mov    %rax,%rbx
   b8623:	cmpq   $0x0,0x60(%rsp)
   b8629:	je     b8640 <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1f40>
   b862b:	mov    $0x1,%edx
   b8630:	mov    0x40(%rsp),%rdi
   b8635:	mov    0x60(%rsp),%rsi
   b863a:	call   *0x20e6a0(%rip)        # 2c6ce0 <_DYNAMIC+0x250>
   b8640:	cmpq   $0x0,(%rsp)
   b8645:	je     b865b <rvvdk_vmdk::stream_map::StreamMap::read_from+0x1f5b>
   b8647:	mov    $0x1,%edx
   b864c:	mov    0x18(%rsp),%rdi
   b8651:	mov    (%rsp),%rsi
   b8655:	call   *0x20e685(%rip)        # 2c6ce0 <_DYNAMIC+0x250>
   b865b:	mov    %rbx,%rdi
   b865e:	call   2b86c0 <_Unwind_Resume@plt>
