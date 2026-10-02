
target/r511b/reference/map-candidate:     file format elf64-x86-64


Disassembly of section .text:

00000000000d1c20 <rvvdk_vmdk::stream_map::StreamMap::grain>:
   d1c20:	mov    %rdi,%rax
   d1c23:	mov    0x28(%rsi),%rcx
   d1c27:	shr    $0x10,%rcx
   d1c2b:	cmp    %rcx,%rdx
   d1c2e:	jae    d1c53 <rvvdk_vmdk::stream_map::StreamMap::grain+0x33>
   d1c30:	mov    0x70(%rsi),%rdi
   d1c34:	xor    %r8d,%r8d
   d1c37:	test   %rdi,%rdi
   d1c3a:	je     d1cbc <rvvdk_vmdk::stream_map::StreamMap::grain+0x9c>
   d1c40:	mov    0x68(%rsi),%rcx
   d1c44:	cmp    $0x1,%rdi
   d1c48:	jne    d1c6e <rvvdk_vmdk::stream_map::StreamMap::grain+0x4e>
   d1c4a:	mov    (%rcx),%esi
   d1c4c:	cmp    %rsi,%rdx
   d1c4f:	je     d1ca7 <rvvdk_vmdk::stream_map::StreamMap::grain+0x87>
   d1c51:	jmp    d1cbc <rvvdk_vmdk::stream_map::StreamMap::grain+0x9c>
   d1c53:	movq   $0xa,(%rax)
   d1c5a:	lea    -0xb0784(%rip),%rcx        # 214dd <anon.2bae575864a295c13be08024f1bafd24.11.llvm.2369629691934005234+0x3d7>
   d1c61:	mov    %rcx,0x8(%rax)
   d1c65:	movq   $0xb,0x10(%rax)
   d1c6d:	ret
   d1c6e:	xor    %esi,%esi
   d1c70:	mov    %rsi,%r8
   d1c73:	mov    %rdi,%r9
   d1c76:	shr    $1,%r9
   d1c79:	add    %r9,%rsi
   d1c7c:	lea    (%rsi,%rsi,2),%r10
   d1c80:	mov    (%rcx,%r10,4),%r10d
   d1c84:	cmp    %r10,%rdx
   d1c87:	cmovb  %r8,%rsi
   d1c8b:	sub    %r9,%rdi
   d1c8e:	cmp    $0x1,%rdi
   d1c92:	ja     d1c70 <rvvdk_vmdk::stream_map::StreamMap::grain+0x50>
   d1c94:	lea    (%rsi,%rsi,2),%rsi
   d1c98:	mov    (%rcx,%rsi,4),%edi
   d1c9b:	xor    %r8d,%r8d
   d1c9e:	cmp    %rdi,%rdx
   d1ca1:	jne    d1cbc <rvvdk_vmdk::stream_map::StreamMap::grain+0x9c>
   d1ca3:	lea    (%rcx,%rsi,4),%rcx
   d1ca7:	mov    0x8(%rcx),%edx
   d1caa:	mov    %edx,-0x8(%rsp)
   d1cae:	mov    (%rcx),%rcx
   d1cb1:	mov    %rcx,-0x10(%rsp)
   d1cb6:	mov    $0x1,%r8d
   d1cbc:	mov    %r8d,0x8(%rax)
   d1cc0:	mov    -0x10(%rsp),%rcx
   d1cc5:	mov    %rcx,0xc(%rax)
   d1cc9:	mov    -0x8(%rsp),%ecx
   d1ccd:	mov    %ecx,0x14(%rax)
   d1cd0:	movq   $0x10,(%rax)
   d1cd7:	ret
