
target/r511a/reference/stream-before:     file format elf64-x86-64


Disassembly of section .text:

00000000000ca9e0 <rvvdk_vmdk::stream::StreamMarker::parse>:
   ca9e0:	mov    %rdi,%rax
   ca9e3:	cmp    $0x10,%rdx
   ca9e7:	jne    caa2d <rvvdk_vmdk::stream::StreamMarker::parse+0x4d>
   ca9e9:	mov    %ecx,%edx
   ca9eb:	and    $0x1ff,%edx
   ca9f1:	jne    caa2d <rvvdk_vmdk::stream::StreamMarker::parse+0x4d>
   ca9f3:	cmp    0x48(%r8),%rcx
   ca9f7:	jb     caa2d <rvvdk_vmdk::stream::StreamMarker::parse+0x4d>
   ca9f9:	mov    (%rsi),%rdx
   ca9fc:	mov    0x8(%rsi),%edi
   ca9ff:	test   %rdi,%rdi
   caa02:	je     caa50 <rvvdk_vmdk::stream::StreamMarker::parse+0x70>
   caa04:	cmp    %rdi,0x20(%r9)
   caa08:	jae    caa80 <rvvdk_vmdk::stream::StreamMarker::parse+0xa0>
   caa0a:	movq   $0xb,0x8(%rax)
   caa12:	lea    -0xa9a05(%rip),%rcx        # 21014 <anon.ca35c66e77da7ac5f12473bd4dd79fb8.4.llvm.7343412667217649661+0x47d>
   caa19:	mov    %rcx,0x10(%rax)
   caa1d:	movq   $0x16,0x18(%rax)
   caa25:	movq   $0x5,(%rax)
   caa2c:	ret
   caa2d:	movq   $0xa,0x8(%rax)
   caa35:	lea    -0xa9a12(%rip),%rcx        # 2102a <anon.ca35c66e77da7ac5f12473bd4dd79fb8.4.llvm.7343412667217649661+0x493>
   caa3c:	mov    %rcx,0x10(%rax)
   caa40:	movq   $0x16,0x18(%rax)
   caa48:	movq   $0x5,(%rax)
   caa4f:	ret
   caa50:	mov    0xc(%rsi),%esi
   caa53:	cmp    $0x3,%rsi
   caa57:	ja     cab6e <rvvdk_vmdk::stream::StreamMarker::parse+0x18e>
   caa5d:	lea    -0xa9c04(%rip),%rdi        # 20e60 <anon.ca35c66e77da7ac5f12473bd4dd79fb8.4.llvm.7343412667217649661+0x2c9>
   caa64:	movslq (%rdi,%rsi,4),%r9
   caa68:	add    %rdi,%r9
   caa6b:	mov    %rsi,%rdi
   caa6e:	jmp    *%r9
   caa71:	mov    $0x4,%edi
   caa76:	cmp    %rdi,%rdx
   caa79:	je     caaf1 <rvvdk_vmdk::stream::StreamMarker::parse+0x111>
   caa7b:	jmp    cab4b <rvvdk_vmdk::stream::StreamMarker::parse+0x16b>
   caa80:	mov    %rdx,%rsi
   caa83:	shr    $0x37,%rsi
   caa87:	jne    cace6 <rvvdk_vmdk::stream::StreamMarker::parse+0x306>
   caa8d:	test   $0x7f,%dl
   caa90:	jne    caac1 <rvvdk_vmdk::stream::StreamMarker::parse+0xe1>
   caa92:	movabs $0x7fffffffffffff,%r9
   caa9c:	mov    %rdx,%rsi
   caa9f:	shl    $0x9,%rsi
   caaa3:	lea    0x10000(%rsi),%r10
   caaaa:	lea    -0x80(%r9),%r11
   caaae:	cmp    %r11,%rdx
   caab1:	ja     cad1e <rvvdk_vmdk::stream::StreamMarker::parse+0x33e>
   caab7:	cmp    0x28(%r8),%r10
   caabb:	jbe    cabde <rvvdk_vmdk::stream::StreamMarker::parse+0x1fe>
   caac1:	movq   $0xa,0x8(%rax)
   caac9:	lea    -0xa9acf(%rip),%rcx        # 21001 <anon.ca35c66e77da7ac5f12473bd4dd79fb8.4.llvm.7343412667217649661+0x46a>
   caad0:	mov    %rcx,0x10(%rax)
   caad4:	movq   $0x13,0x18(%rax)
   caadc:	movq   $0x5,(%rax)
   caae3:	ret
   caae4:	mov    0x50(%r8),%rdi
   caae8:	shr    $0x9,%rdi
   caaec:	cmp    %rdi,%rdx
   caaef:	jne    cab4b <rvvdk_vmdk::stream::StreamMarker::parse+0x16b>
   caaf1:	lea    0x200(%rcx),%rdi
   caaf8:	cmp    $0xfffffffffffffdff,%rcx
   caaff:	ja     cacf6 <rvvdk_vmdk::stream::StreamMarker::parse+0x316>
   cab05:	shl    $0x9,%rdx
   cab09:	mov    %rdx,%rcx
   cab0c:	add    %rdi,%rcx
   cab0f:	jb     cad0a <rvvdk_vmdk::stream::StreamMarker::parse+0x32a>
   cab15:	mov    0x30(%r8),%r9
   cab19:	cmp    %r9,%rcx
   cab1c:	jbe    cab91 <rvvdk_vmdk::stream::StreamMarker::parse+0x1b1>
   cab1e:	movq   $0xa,0x8(%rax)
   cab26:	lea    -0xa9b73(%rip),%rcx        # 20fba <anon.ca35c66e77da7ac5f12473bd4dd79fb8.4.llvm.7343412667217649661+0x423>
   cab2d:	mov    %rcx,0x10(%rax)
   cab31:	movq   $0x15,0x18(%rax)
   cab39:	movq   $0x5,(%rax)
   cab40:	ret
   cab41:	mov    $0x1,%edi
   cab46:	cmp    %rdi,%rdx
   cab49:	je     caaf1 <rvvdk_vmdk::stream::StreamMarker::parse+0x111>
   cab4b:	movq   $0xa,0x8(%rax)
   cab53:	lea    -0xa9b8b(%rip),%rcx        # 20fcf <anon.ca35c66e77da7ac5f12473bd4dd79fb8.4.llvm.7343412667217649661+0x438>
   cab5a:	mov    %rcx,0x10(%rax)
   cab5e:	movq   $0x13,0x18(%rax)
   cab66:	movq   $0x5,(%rax)
   cab6d:	ret
   cab6e:	movq   $0x9,0x8(%rax)
   cab76:	lea    -0xa9b9b(%rip),%rcx        # 20fe2 <anon.ca35c66e77da7ac5f12473bd4dd79fb8.4.llvm.7343412667217649661+0x44b>
   cab7d:	mov    %rcx,0x10(%rax)
   cab81:	movq   $0xb,0x18(%rax)
   cab89:	movq   $0x5,(%rax)
   cab90:	ret
   cab91:	lea    -0x1(%rsi),%r10d
   cab95:	cmp    $0x2,%r10d
   cab99:	jae    cac4c <rvvdk_vmdk::stream::StreamMarker::parse+0x26c>
   cab9f:	add    $0xfffffffffffffa00,%r9
   caba6:	cmp    %r9,%rcx
   caba9:	jbe    cac75 <rvvdk_vmdk::stream::StreamMarker::parse+0x295>
   cabaf:	mov    (%r8),%rcx
   cabb2:	cmp    $0x2,%ecx
   cabb5:	jne    cac75 <rvvdk_vmdk::stream::StreamMarker::parse+0x295>
   cabbb:	movq   $0xa,0x8(%rax)
   cabc3:	lea    -0xa9c4a(%rip),%rcx        # 20f80 <anon.ca35c66e77da7ac5f12473bd4dd79fb8.4.llvm.7343412667217649661+0x3e9>
   cabca:	mov    %rcx,0x10(%rax)
   cabce:	movq   $0x22,0x18(%rax)
   cabd6:	movq   $0x5,(%rax)
   cabdd:	ret
   cabde:	or     $0xc,%rcx
   cabe2:	mov    %rcx,%r10
   cabe5:	add    %rdi,%r10
   cabe8:	jb     cad32 <rvvdk_vmdk::stream::StreamMarker::parse+0x352>
   cabee:	mov    %r10,%rdx
   cabf1:	shr    $0x9,%rdx
   cabf5:	and    $0x1ff,%r10d
   cabfc:	cmp    $0x1,%r10
   cac00:	sbb    $0xffffffffffffffff,%rdx
   cac04:	cmp    %r9,%rdx
   cac07:	ja     cad4b <rvvdk_vmdk::stream::StreamMarker::parse+0x36b>
   cac0d:	shl    $0x9,%rdx
   cac11:	mov    0x30(%r8),%r9
   cac15:	cmpl   $0x2,(%r8)
   cac19:	lea    -0x600(%r9),%r8
   cac20:	cmovne %r9,%r8
   cac24:	cmp    %r8,%rdx
   cac27:	jbe    cac5d <rvvdk_vmdk::stream::StreamMarker::parse+0x27d>
   cac29:	movq   $0xa,0x8(%rax)
   cac31:	lea    -0xa9c4b(%rip),%rcx        # 20fed <anon.ca35c66e77da7ac5f12473bd4dd79fb8.4.llvm.7343412667217649661+0x456>
   cac38:	mov    %rcx,0x10(%rax)
   cac3c:	movq   $0x14,0x18(%rax)
   cac44:	movq   $0x5,(%rax)
   cac4b:	ret
   cac4c:	test   %esi,%esi
   cac4e:	jne    cac8a <rvvdk_vmdk::stream::StreamMarker::parse+0x2aa>
   cac50:	cmp    %r9,%rdi
   cac53:	jne    cacb3 <rvvdk_vmdk::stream::StreamMarker::parse+0x2d3>
   cac55:	movq   $0x4,(%rax)
   cac5c:	ret
   cac5d:	movq   $0x0,(%rax)
   cac64:	mov    %rsi,0x8(%rax)
   cac68:	mov    %rdx,0x10(%rax)
   cac6c:	mov    %rcx,0x18(%rax)
   cac70:	mov    %rdi,0x20(%rax)
   cac74:	ret
   cac75:	cmp    $0x2,%esi
   cac78:	jne    cacd6 <rvvdk_vmdk::stream::StreamMarker::parse+0x2f6>
   cac7a:	movq   $0x2,(%rax)
   cac81:	mov    %rdi,0x8(%rax)
   cac85:	mov    %rdx,0x10(%rax)
   cac89:	ret
   cac8a:	lea    0x200(%rcx),%rsi
   cac91:	cmp    $0xfffffffffffffdff,%rcx
   cac98:	ja     cad62 <rvvdk_vmdk::stream::StreamMarker::parse+0x382>
   cac9e:	cmp    %r9,%rsi
   caca1:	jne    cacb3 <rvvdk_vmdk::stream::StreamMarker::parse+0x2d3>
   caca3:	movq   $0x3,(%rax)
   cacaa:	mov    %rdi,0x8(%rax)
   cacae:	mov    %rdx,0x10(%rax)
   cacb2:	ret
   cacb3:	movq   $0xa,0x8(%rax)
   cacbb:	lea    -0xa9d20(%rip),%rcx        # 20fa2 <anon.ca35c66e77da7ac5f12473bd4dd79fb8.4.llvm.7343412667217649661+0x40b>
   cacc2:	mov    %rcx,0x10(%rax)
   cacc6:	movq   $0x18,0x18(%rax)
   cacce:	movq   $0x5,(%rax)
   cacd5:	ret
   cacd6:	movq   $0x1,(%rax)
   cacdd:	mov    %rdi,0x8(%rax)
   cace1:	mov    %rdx,0x10(%rax)
   cace5:	ret
   cace6:	movq   $0xc,0x8(%rax)
   cacee:	movq   $0x5,(%rax)
   cacf5:	ret
   cacf6:	movq   $0xc,0x8(%rax)
   cacfe:	mov    %rdi,0x10(%rax)
   cad02:	movq   $0x5,(%rax)
   cad09:	ret
   cad0a:	movq   $0xc,0x8(%rax)
   cad12:	mov    %rcx,0x10(%rax)
   cad16:	movq   $0x5,(%rax)
   cad1d:	ret
   cad1e:	movq   $0xc,0x8(%rax)
   cad26:	mov    %r10,0x10(%rax)
   cad2a:	movq   $0x5,(%rax)
   cad31:	ret
   cad32:	movq   $0xc,0x8(%rax)
   cad3a:	mov    $0x10,%ecx
   cad3f:	mov    %r10,(%rax,%rcx,1)
   cad43:	movq   $0x5,(%rax)
   cad4a:	ret
   cad4b:	mov    $0xc,%r10d
   cad51:	mov    $0x8,%ecx
   cad56:	mov    %r10,(%rax,%rcx,1)
   cad5a:	movq   $0x5,(%rax)
   cad61:	ret
   cad62:	movq   $0xc,0x8(%rax)
   cad6a:	mov    %rsi,0x10(%rax)
   cad6e:	movq   $0x5,(%rax)
   cad75:	ret
