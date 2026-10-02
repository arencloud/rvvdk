
target/r510p/reference/stream-before:     file format elf64-x86-64


Disassembly of section .text:

00000000000c97e0 <rvvdk_vmdk::stream::StreamHeader::parse_inner>:
   c97e0:	mov    %rdi,%rax
   c97e3:	cmp    $0x200,%rdx
   c97ea:	jne    c9869 <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x89>
   c97ec:	cmpl   $0x564d444b,(%rsi)
   c97f2:	jne    c988c <rvvdk_vmdk::stream::StreamHeader::parse_inner+0xac>
   c97f8:	cmpl   $0x3,0x4(%rsi)
   c97fc:	jne    c98af <rvvdk_vmdk::stream::StreamHeader::parse_inner+0xcf>
   c9802:	mov    0x8(%rsi),%edx
   c9805:	mov    %edx,%edi
   c9807:	and    $0xfffffffc,%edi
   c980a:	cmp    $0x30000,%edi
   c9810:	jne    c98d2 <rvvdk_vmdk::stream::StreamHeader::parse_inner+0xf2>
   c9816:	cmpq   $0x80,0x14(%rsi)
   c981e:	jne    c98f5 <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x115>
   c9824:	cmpl   $0x200,0x2c(%rsi)
   c982b:	jne    c98f5 <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x115>
   c9831:	cmpw   $0x1,0x4d(%rsi)
   c9836:	jne    c9918 <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x138>
   c983c:	cmpb   $0x0,0x48(%rsi)
   c9840:	je     c993b <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x15b>
   c9846:	movq   $0x9,0x8(%rax)
   c984e:	lea    -0xb2565(%rip),%rcx        # 172f0 <anon.d625489d584c397ac22d75864c33158f.37.llvm.5936746164385555759+0x80>
   c9855:	mov    %rcx,0x10(%rax)
   c9859:	movq   $0x10,0x18(%rax)
   c9861:	movq   $0x3,(%rax)
   c9868:	ret
   c9869:	movq   $0xa,0x8(%rax)
   c9871:	lea    -0xa89ad(%rip),%rcx        # 20ecb <anon.ca35c66e77da7ac5f12473bd4dd79fb8.4.llvm.7343412667217649661+0x3ac>
   c9878:	mov    %rcx,0x10(%rax)
   c987c:	movq   $0xd,0x18(%rax)
   c9884:	movq   $0x3,(%rax)
   c988b:	ret
   c988c:	movq   $0xa,0x8(%rax)
   c9894:	lea    -0xa89d5(%rip),%rcx        # 20ec6 <anon.ca35c66e77da7ac5f12473bd4dd79fb8.4.llvm.7343412667217649661+0x3a7>
   c989b:	mov    %rcx,0x10(%rax)
   c989f:	movq   $0x5,0x18(%rax)
   c98a7:	movq   $0x3,(%rax)
   c98ae:	ret
   c98af:	movq   $0x9,0x8(%rax)
   c98b7:	lea    -0xb27de(%rip),%rcx        # 170e0 <anon.f8fa127e3698a51bcdecbcd58237901a.58.llvm.13641424853953997931+0x40>
   c98be:	mov    %rcx,0x10(%rax)
   c98c2:	movq   $0x10,0x18(%rax)
   c98ca:	movq   $0x3,(%rax)
   c98d1:	ret
   c98d2:	movq   $0x9,0x8(%rax)
   c98da:	lea    -0xa8a20(%rip),%rcx        # 20ec1 <anon.ca35c66e77da7ac5f12473bd4dd79fb8.4.llvm.7343412667217649661+0x3a2>
   c98e1:	mov    %rcx,0x10(%rax)
   c98e5:	movq   $0x5,0x18(%rax)
   c98ed:	movq   $0x3,(%rax)
   c98f4:	ret
   c98f5:	movq   $0x9,0x8(%rax)
   c98fd:	lea    -0xa8a57(%rip),%rcx        # 20ead <anon.ca35c66e77da7ac5f12473bd4dd79fb8.4.llvm.7343412667217649661+0x38e>
   c9904:	mov    %rcx,0x10(%rax)
   c9908:	movq   $0x14,0x18(%rax)
   c9910:	movq   $0x3,(%rax)
   c9917:	ret
   c9918:	movq   $0x9,0x8(%rax)
   c9920:	lea    -0xa8a8f(%rip),%rcx        # 20e98 <anon.ca35c66e77da7ac5f12473bd4dd79fb8.4.llvm.7343412667217649661+0x379>
   c9927:	mov    %rcx,0x10(%rax)
   c992b:	movq   $0x15,0x18(%rax)
   c9933:	movq   $0x3,(%rax)
   c993a:	ret
   c993b:	test   $0x1,%dl
   c993e:	je     c9949 <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x169>
   c9940:	cmpl   $0xa0d200a,0x49(%rsi)
   c9947:	jne    c998b <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x1ab>
   c9949:	mov    $0x50,%edi
   c994e:	cmpb   $0x0,-0x1(%rsi,%rdi,1)
   c9953:	jne    c9968 <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x188>
   c9955:	cmp    $0x200,%rdi
   c995c:	je     c99ae <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x1ce>
   c995e:	cmpb   $0x0,(%rsi,%rdi,1)
   c9962:	lea    0x2(%rdi),%rdi
   c9966:	je     c994e <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x16e>
   c9968:	movq   $0x9,0x8(%rax)
   c9970:	lea    -0xa8b01(%rip),%rcx        # 20e76 <anon.ca35c66e77da7ac5f12473bd4dd79fb8.4.llvm.7343412667217649661+0x357>
   c9977:	mov    %rcx,0x10(%rax)
   c997b:	movq   $0x15,0x18(%rax)
   c9983:	movq   $0x3,(%rax)
   c998a:	ret
   c998b:	movq   $0xa,0x8(%rax)
   c9993:	lea    -0xa8b0f(%rip),%rcx        # 20e8b <anon.ca35c66e77da7ac5f12473bd4dd79fb8.4.llvm.7343412667217649661+0x36c>
   c999a:	mov    %rcx,0x10(%rax)
   c999e:	movq   $0xd,0x18(%rax)
   c99a6:	movq   $0x3,(%rax)
   c99ad:	ret
   c99ae:	cmp    $0x400,%rcx
   c99b5:	setae  %dil
   c99b9:	test   $0x1ff,%ecx
   c99bf:	sete   %r10b
   c99c3:	test   %r10b,%dil
   c99c6:	je     c99f1 <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x211>
   c99c8:	cmp    0x8(%r8),%rcx
   c99cc:	jbe    c9a14 <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x234>
   c99ce:	movq   $0xb,0x8(%rax)
   c99d6:	lea    -0xa8b80(%rip),%rcx        # 20e5d <anon.ca35c66e77da7ac5f12473bd4dd79fb8.4.llvm.7343412667217649661+0x33e>
   c99dd:	mov    %rcx,0x10(%rax)
   c99e1:	movq   $0xc,0x18(%rax)
   c99e9:	movq   $0x3,(%rax)
   c99f0:	ret
   c99f1:	movq   $0xa,0x8(%rax)
   c99f9:	lea    -0xa8b97(%rip),%rcx        # 20e69 <anon.ca35c66e77da7ac5f12473bd4dd79fb8.4.llvm.7343412667217649661+0x34a>
   c9a00:	mov    %rcx,0x10(%rax)
   c9a04:	movq   $0xd,0x18(%rax)
   c9a0c:	movq   $0x3,(%rax)
   c9a13:	ret
   c9a14:	mov    0xc(%rsi),%r11
   c9a18:	mov    %r11,%rdi
   c9a1b:	shr    $0x37,%rdi
   c9a1f:	jne    c9d07 <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x527>
   c9a25:	test   %r11,%r11
   c9a28:	setne  %dil
   c9a2c:	test   $0x7f,%r11b
   c9a30:	sete   %r10b
   c9a34:	test   %r10b,%dil
   c9a37:	je     c9a68 <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x288>
   c9a39:	mov    %r11,%r10
   c9a3c:	shl    $0x9,%r10
   c9a40:	cmp    (%r8),%r10
   c9a43:	jbe    c9a8b <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x2ab>
   c9a45:	movq   $0xb,0x8(%rax)
   c9a4d:	lea    -0xa8c05(%rip),%rcx        # 20e4f <anon.ca35c66e77da7ac5f12473bd4dd79fb8.4.llvm.7343412667217649661+0x330>
   c9a54:	mov    %rcx,0x10(%rax)
   c9a58:	movq   $0xe,0x18(%rax)
   c9a60:	movq   $0x3,(%rax)
   c9a67:	ret
   c9a68:	movq   $0xa,0x8(%rax)
   c9a70:	lea    -0xb23e7(%rip),%rcx        # 17690 <anon.786f9ca1aa6bf68944dde0bec4ae4f7b.2.llvm.4745231054059156856+0x300>
   c9a77:	mov    %rcx,0x10(%rax)
   c9a7b:	movq   $0x8,0x18(%rax)
   c9a83:	movq   $0x3,(%rax)
   c9a8a:	ret
   c9a8b:	push   %rbp
   c9a8c:	push   %r15
   c9a8e:	push   %r14
   c9a90:	push   %r13
   c9a92:	push   %r12
   c9a94:	push   %rbx
   c9a95:	mov    %r11,%r14
   c9a98:	shr    $0x10,%r14
   c9a9c:	and    $0xff80,%r11d
   c9aa3:	cmp    $0x1,%r11
   c9aa7:	sbb    $0xffffffffffffffff,%r14
   c9aab:	cmp    0x18(%r8),%r14
   c9aaf:	jbe    c9ad1 <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x2f1>
   c9ab1:	movq   $0xb,0x8(%rax)
   c9ab9:	lea    -0xa8c82(%rip),%rcx        # 20e3e <anon.ca35c66e77da7ac5f12473bd4dd79fb8.4.llvm.7343412667217649661+0x31f>
   c9ac0:	mov    %rcx,0x10(%rax)
   c9ac4:	movq   $0x11,0x18(%rax)
   c9acc:	jmp    c9bcc <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x3ec>
   c9ad1:	mov    %rcx,%rdi
   c9ad4:	movabs $0x7fffffffffffff,%r13
   c9ade:	mov    0x40(%rsi),%rcx
   c9ae2:	cmp    %r13,%rcx
   c9ae5:	ja     c9dac <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x5cc>
   c9aeb:	mov    %rcx,%r11
   c9aee:	shl    $0x9,%r11
   c9af2:	test   $0x7f,%cl
   c9af5:	sete   %cl
   c9af8:	lea    -0x1(%r11),%rbx
   c9afc:	cmp    %rdi,%rbx
   c9aff:	setb   %bl
   c9b02:	test   %bl,%cl
   c9b04:	jne    c9b26 <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x346>
   c9b06:	movq   $0xa,0x8(%rax)
   c9b0e:	lea    -0xb224d(%rip),%rcx        # 178c8 <anon.76985b85afb607e444f230055e37566f.24.llvm.7116865095658657516+0x30>
   c9b15:	mov    %rcx,0x10(%rax)
   c9b19:	movq   $0x8,0x18(%rax)
   c9b21:	jmp    c9bcc <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x3ec>
   c9b26:	cmp    0x10(%r8),%r11
   c9b2a:	jbe    c9b4c <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x36c>
   c9b2c:	movq   $0xb,0x8(%rax)
   c9b34:	lea    -0xa8d0b(%rip),%rcx        # 20e30 <anon.ca35c66e77da7ac5f12473bd4dd79fb8.4.llvm.7343412667217649661+0x311>
   c9b3b:	mov    %rcx,0x10(%rax)
   c9b3f:	movq   $0xe,0x18(%rax)
   c9b47:	jmp    c9bcc <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x3ec>
   c9b4c:	mov    0x24(%rsi),%r12
   c9b50:	cmp    %r13,%r12
   c9b53:	ja     c9dac <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x5cc>
   c9b59:	mov    %r12,%rbx
   c9b5c:	shl    $0x9,%rbx
   c9b60:	cmp    0x30(%r8),%rbx
   c9b64:	jbe    c9b83 <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x3a3>
   c9b66:	movq   $0xb,0x8(%rax)
   c9b6e:	lea    -0xb2a55(%rip),%rcx        # 17120 <anon.f8fa127e3698a51bcdecbcd58237901a.58.llvm.13641424853953997931+0x80>
   c9b75:	mov    %rcx,0x10(%rax)
   c9b79:	movq   $0x10,0x18(%rax)
   c9b81:	jmp    c9bcc <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x3ec>
   c9b83:	mov    0x1c(%rsi),%r8
   c9b87:	cmp    %r13,%r8
   c9b8a:	ja     c9e47 <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x667>
   c9b90:	mov    $0xa,%r15d
   c9b96:	lea    -0xa8c05(%rip),%rcx        # 20f98 <anon.ca35c66e77da7ac5f12473bd4dd79fb8.4.llvm.7343412667217649661+0x479>
   c9b9d:	test   %r12,%r12
   c9ba0:	je     c9bbc <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x3dc>
   c9ba2:	test   %r8,%r8
   c9ba5:	je     c9bbc <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x3dc>
   c9ba7:	shl    $0x9,%r8
   c9bab:	mov    %r8,%rbp
   c9bae:	add    %rbx,%rbp
   c9bb1:	jb     c9e52 <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x672>
   c9bb7:	cmp    %r11,%rbp
   c9bba:	jbe    c9bde <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x3fe>
   c9bbc:	mov    %r15,0x8(%rax)
   c9bc0:	mov    %rcx,0x10(%rax)
   c9bc4:	movq   $0xd,0x18(%rax)
   c9bcc:	movq   $0x3,(%rax)
   c9bd3:	pop    %rbx
   c9bd4:	pop    %r12
   c9bd6:	pop    %r13
   c9bd8:	pop    %r14
   c9bda:	pop    %r15
   c9bdc:	pop    %rbp
   c9bdd:	ret
   c9bde:	lea    0x1fc(,%r14,4),%rcx
   c9be6:	movabs $0x7fffffffe00,%r15
   c9bf0:	and    %rcx,%r15
   c9bf3:	mov    %r15,-0x10(%rsp)
   c9bf8:	mov    0x38(%rsi),%r15
   c9bfc:	cmp    $0xffffffffffffffff,%r15
   c9c00:	je     c9c2c <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x44c>
   c9c02:	mov    %r11,%rcx
   c9c05:	test   %r9b,%r9b
   c9c08:	je     c9cb0 <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x4d0>
   c9c0e:	cmp    $0x600,%rdi
   c9c15:	jae    c9ca9 <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x4c9>
   c9c1b:	movq   $0xa,0x8(%rax)
   c9c23:	lea    -0xa8e07(%rip),%rcx        # 20e23 <anon.ca35c66e77da7ac5f12473bd4dd79fb8.4.llvm.7343412667217649661+0x304>
   c9c2a:	jmp    c9bc0 <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x3e0>
   c9c2c:	mov    %edx,%ecx
   c9c2e:	and    $0x2,%ecx
   c9c31:	shr    $1,%ecx
   c9c33:	or     %cl,%r9b
   c9c36:	mov    %r11,%rcx
   c9c39:	or     $0x600,%rcx
   c9c40:	cmp    %rdi,%rcx
   c9c43:	seta   %cl
   c9c46:	or     %r9b,%cl
   c9c49:	je     c9c6b <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x48b>
   c9c4b:	movq   $0xa,0x8(%rax)
   c9c53:	lea    -0xa8e92(%rip),%rcx        # 20dc8 <anon.ca35c66e77da7ac5f12473bd4dd79fb8.4.llvm.7343412667217649661+0x2a9>
   c9c5a:	mov    %rcx,0x10(%rax)
   c9c5e:	movq   $0x18,0x18(%rax)
   c9c66:	jmp    c9bcc <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x3ec>
   c9c6b:	mov    $0x2,%r9d
   c9c71:	mov    %r9,(%rax)
   c9c74:	mov    %rsi,0x8(%rax)
   c9c78:	mov    -0x10(%rsp),%rcx
   c9c7d:	mov    %rcx,0x10(%rax)
   c9c81:	mov    %r15,0x18(%rax)
   c9c85:	mov    %rcx,0x20(%rax)
   c9c89:	mov    %r10,0x28(%rax)
   c9c8d:	mov    %rdi,0x30(%rax)
   c9c91:	mov    %r8,0x38(%rax)
   c9c95:	mov    %rbx,0x40(%rax)
   c9c99:	mov    %r11,0x48(%rax)
   c9c9d:	mov    %rcx,0x50(%rax)
   c9ca1:	mov    %edx,0x58(%rax)
   c9ca4:	jmp    c9bd3 <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x3f3>
   c9ca9:	lea    -0x600(%rdi),%rcx
   c9cb0:	cmp    %r13,%r15
   c9cb3:	ja     c9e60 <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x680>
   c9cb9:	mov    $0xa,%r12d
   c9cbf:	mov    %r12,-0x18(%rsp)
   c9cc4:	lea    -0xa8d33(%rip),%r12        # 20f98 <anon.ca35c66e77da7ac5f12473bd4dd79fb8.4.llvm.7343412667217649661+0x479>
   c9ccb:	mov    %r12,-0x8(%rsp)
   c9cd0:	cmpq   $0x0,-0x10(%rsp)
   c9cd6:	je     c9cf4 <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x514>
   c9cd8:	test   %r15,%r15
   c9cdb:	je     c9cf4 <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x514>
   c9cdd:	shl    $0x9,%r15
   c9ce1:	mov    %r15,%r12
   c9ce4:	add    -0x10(%rsp),%r12
   c9ce9:	jb     c9e62 <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x682>
   c9cef:	cmp    %rcx,%r12
   c9cf2:	jbe    c9d17 <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x537>
   c9cf4:	mov    -0x18(%rsp),%rcx
   c9cf9:	mov    %rcx,0x8(%rax)
   c9cfd:	mov    -0x8(%rsp),%rcx
   c9d02:	jmp    c9bc0 <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x3e0>
   c9d07:	movq   $0xc,0x8(%rax)
   c9d0f:	movq   $0x3,(%rax)
   c9d16:	ret
   c9d17:	cmp    %rbp,%r15
   c9d1a:	jae    c9d21 <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x541>
   c9d1c:	cmp    %r12,%r8
   c9d1f:	jb     c9d3c <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x55c>
   c9d21:	test   %r9b,%r9b
   c9d24:	je     c9d5c <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x57c>
   c9d26:	cmp    %r11,%r15
   c9d29:	jbe    c9d3c <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x55c>
   c9d2b:	test   $0x2,%dl
   c9d2e:	jne    c9e05 <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x625>
   c9d34:	xor    %r9d,%r9d
   c9d37:	jmp    c9c71 <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x491>
   c9d3c:	movq   $0xa,0x8(%rax)
   c9d44:	lea    -0xa8f42(%rip),%rcx        # 20e09 <anon.ca35c66e77da7ac5f12473bd4dd79fb8.4.llvm.7343412667217649661+0x2ea>
   c9d4b:	mov    %rcx,0x10(%rax)
   c9d4f:	movq   $0x1a,0x18(%rax)
   c9d57:	jmp    c9bcc <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x3ec>
   c9d5c:	mov    %rdx,%rcx
   c9d5f:	and    $0x2,%rcx
   c9d63:	mov    %rcx,-0x18(%rsp)
   c9d68:	je     c9db9 <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x5d9>
   c9d6a:	mov    0x30(%rsi),%rsi
   c9d6e:	cmp    %r13,%rsi
   c9d71:	ja     c9e76 <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x696>
   c9d77:	mov    $0xa,%ecx
   c9d7c:	lea    -0xa8deb(%rip),%r9        # 20f98 <anon.ca35c66e77da7ac5f12473bd4dd79fb8.4.llvm.7343412667217649661+0x479>
   c9d83:	test   %rsi,%rsi
   c9d86:	je     c9d9f <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x5bf>
   c9d88:	shl    $0x9,%rsi
   c9d8c:	mov    %rsi,%r13
   c9d8f:	add    -0x10(%rsp),%r13
   c9d94:	jb     c9ea6 <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x6c6>
   c9d9a:	cmp    %r11,%r13
   c9d9d:	jbe    c9e19 <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x639>
   c9d9f:	mov    %rcx,0x8(%rax)
   c9da3:	mov    %r9,0x10(%rax)
   c9da7:	jmp    c9bc4 <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x3e4>
   c9dac:	movq   $0xc,0x8(%rax)
   c9db4:	jmp    c9bcc <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x3ec>
   c9db9:	xor    %r9d,%r9d
   c9dbc:	mov    %rbx,%r13
   c9dbf:	add    $0x200,%r13
   c9dc6:	je     c9e80 <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x6a0>
   c9dcc:	shl    $0xb,%r14
   c9dd0:	add    -0x10(%rsp),%r14
   c9dd5:	mov    -0x18(%rsp),%rcx
   c9dda:	shr    $1,%ecx
   c9ddc:	shl    %cl,%r14
   c9ddf:	add    %r14,%r13
   c9de2:	jb     c9e95 <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x6b5>
   c9de8:	cmp    %r11,%r13
   c9deb:	jbe    c9c71 <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x491>
   c9df1:	movq   $0xa,0x8(%rax)
   c9df9:	lea    -0xa9020(%rip),%rcx        # 20de0 <anon.ca35c66e77da7ac5f12473bd4dd79fb8.4.llvm.7343412667217649661+0x2c1>
   c9e00:	jmp    c9c5a <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x47a>
   c9e05:	movq   $0xa,0x8(%rax)
   c9e0d:	lea    -0xb2b34(%rip),%rcx        # 172e0 <anon.d625489d584c397ac22d75864c33158f.37.llvm.5936746164385555759+0x70>
   c9e14:	jmp    c9b75 <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x395>
   c9e19:	cmp    %r12,%rsi
   c9e1c:	jae    c9e23 <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x643>
   c9e1e:	cmp    %r13,%r15
   c9e21:	jb     c9e33 <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x653>
   c9e23:	mov    $0x1,%r9d
   c9e29:	cmp    %rbp,%rsi
   c9e2c:	jae    c9dbc <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x5dc>
   c9e2e:	cmp    %r13,%r8
   c9e31:	jae    c9dbc <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x5dc>
   c9e33:	movq   $0xa,0x8(%rax)
   c9e3b:	lea    -0xa904a(%rip),%rcx        # 20df8 <anon.ca35c66e77da7ac5f12473bd4dd79fb8.4.llvm.7343412667217649661+0x2d9>
   c9e42:	jmp    c9ac0 <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x2e0>
   c9e47:	mov    $0xc,%r15d
   c9e4d:	jmp    c9bbc <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x3dc>
   c9e52:	mov    %rbp,%rcx
   c9e55:	mov    $0xc,%r15d
   c9e5b:	jmp    c9bbc <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x3dc>
   c9e60:	jmp    c9e67 <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x687>
   c9e62:	mov    %r12,-0x8(%rsp)
   c9e67:	mov    $0xc,%ecx
   c9e6c:	mov    %rcx,-0x18(%rsp)
   c9e71:	jmp    c9cf4 <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x514>
   c9e76:	mov    $0xc,%ecx
   c9e7b:	jmp    c9d9f <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x5bf>
   c9e80:	movq   $0xc,0x8(%rax)
   c9e88:	movq   $0x0,0x10(%rax)
   c9e90:	jmp    c9bcc <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x3ec>
   c9e95:	movq   $0xc,0x8(%rax)
   c9e9d:	mov    %r13,0x10(%rax)
   c9ea1:	jmp    c9bcc <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x3ec>
   c9ea6:	mov    %r13,%r9
   c9ea9:	mov    $0xc,%ecx
   c9eae:	jmp    c9d9f <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x5bf>
