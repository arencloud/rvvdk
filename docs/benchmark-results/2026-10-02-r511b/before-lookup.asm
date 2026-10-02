
target/r511b/reference/map-before:     file format elf64-x86-64


Disassembly of section .text:

00000000000d2460 <rvvdk_vmdk::stream_map::StreamMap::grain>:
   d2460:	mov    %rdi,%rax
   d2463:	mov    0x28(%rsi),%rcx
   d2467:	shr    $0x10,%rcx
   d246b:	cmp    %rcx,%rdx
   d246e:	jae    d2493 <rvvdk_vmdk::stream_map::StreamMap::grain+0x33>
   d2470:	mov    0x70(%rsi),%rdi
   d2474:	xor    %r8d,%r8d
   d2477:	test   %rdi,%rdi
   d247a:	je     d24fc <rvvdk_vmdk::stream_map::StreamMap::grain+0x9c>
   d2480:	mov    0x68(%rsi),%rcx
   d2484:	cmp    $0x1,%rdi
   d2488:	jne    d24ae <rvvdk_vmdk::stream_map::StreamMap::grain+0x4e>
   d248a:	mov    (%rcx),%esi
   d248c:	cmp    %rsi,%rdx
   d248f:	je     d24e7 <rvvdk_vmdk::stream_map::StreamMap::grain+0x87>
   d2491:	jmp    d24fc <rvvdk_vmdk::stream_map::StreamMap::grain+0x9c>
   d2493:	movq   $0xa,(%rax)
   d249a:	lea    -0xb0dd4(%rip),%rcx        # 216cd <anon.d7379f816619210e71e4b9d57ed96063.11.llvm.8654761215334642747+0x5c7>
   d24a1:	mov    %rcx,0x8(%rax)
   d24a5:	movq   $0xb,0x10(%rax)
   d24ad:	ret
   d24ae:	xor    %esi,%esi
   d24b0:	mov    %rsi,%r8
   d24b3:	mov    %rdi,%r9
   d24b6:	shr    $1,%r9
   d24b9:	add    %r9,%rsi
   d24bc:	lea    (%rsi,%rsi,2),%r10
   d24c0:	mov    (%rcx,%r10,4),%r10d
   d24c4:	cmp    %r10,%rdx
   d24c7:	cmovb  %r8,%rsi
   d24cb:	sub    %r9,%rdi
   d24ce:	cmp    $0x1,%rdi
   d24d2:	ja     d24b0 <rvvdk_vmdk::stream_map::StreamMap::grain+0x50>
   d24d4:	lea    (%rsi,%rsi,2),%rsi
   d24d8:	mov    (%rcx,%rsi,4),%edi
   d24db:	xor    %r8d,%r8d
   d24de:	cmp    %rdi,%rdx
   d24e1:	jne    d24fc <rvvdk_vmdk::stream_map::StreamMap::grain+0x9c>
   d24e3:	lea    (%rcx,%rsi,4),%rcx
   d24e7:	mov    0x8(%rcx),%edx
   d24ea:	mov    %edx,-0x8(%rsp)
   d24ee:	mov    (%rcx),%rcx
   d24f1:	mov    %rcx,-0x10(%rsp)
   d24f6:	mov    $0x1,%r8d
   d24fc:	mov    %r8d,0x8(%rax)
   d2500:	mov    -0x10(%rsp),%rcx
   d2505:	mov    %rcx,0xc(%rax)
   d2509:	mov    -0x8(%rsp),%ecx
   d250d:	mov    %ecx,0x14(%rax)
   d2510:	movq   $0x10,(%rax)
   d2517:	ret
