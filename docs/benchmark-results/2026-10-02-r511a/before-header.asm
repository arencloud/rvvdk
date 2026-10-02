
target/r511a/reference/stream-before:     file format elf64-x86-64


Disassembly of section .text:

00000000000ca2f0 <rvvdk_vmdk::stream::StreamHeader::parse_inner>:
   ca2f0:	mov    %rdi,%rax
   ca2f3:	cmp    $0x200,%rdx
   ca2fa:	jne    ca379 <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x89>
   ca2fc:	cmpl   $0x564d444b,(%rsi)
   ca302:	jne    ca39c <rvvdk_vmdk::stream::StreamHeader::parse_inner+0xac>
   ca308:	cmpl   $0x3,0x4(%rsi)
   ca30c:	jne    ca3bf <rvvdk_vmdk::stream::StreamHeader::parse_inner+0xcf>
   ca312:	mov    0x8(%rsi),%edx
   ca315:	mov    %edx,%edi
   ca317:	and    $0xfffffffc,%edi
   ca31a:	cmp    $0x30000,%edi
   ca320:	jne    ca3e2 <rvvdk_vmdk::stream::StreamHeader::parse_inner+0xf2>
   ca326:	cmpq   $0x80,0x14(%rsi)
   ca32e:	jne    ca405 <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x115>
   ca334:	cmpl   $0x200,0x2c(%rsi)
   ca33b:	jne    ca405 <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x115>
   ca341:	cmpw   $0x1,0x4d(%rsi)
   ca346:	jne    ca428 <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x138>
   ca34c:	cmpb   $0x0,0x48(%rsi)
   ca350:	je     ca44b <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x15b>
   ca356:	movq   $0x9,0x8(%rax)
   ca35e:	lea    -0xb2ff5(%rip),%rcx        # 17370 <anon.d625489d584c397ac22d75864c33158f.37.llvm.5936746164385555759+0x80>
   ca365:	mov    %rcx,0x10(%rax)
   ca369:	movq   $0x10,0x18(%rax)
   ca371:	movq   $0x3,(%rax)
   ca378:	ret
   ca379:	movq   $0xa,0x8(%rax)
   ca381:	lea    -0xa9415(%rip),%rcx        # 20f73 <anon.ca35c66e77da7ac5f12473bd4dd79fb8.4.llvm.7343412667217649661+0x3dc>
   ca388:	mov    %rcx,0x10(%rax)
   ca38c:	movq   $0xd,0x18(%rax)
   ca394:	movq   $0x3,(%rax)
   ca39b:	ret
   ca39c:	movq   $0xa,0x8(%rax)
   ca3a4:	lea    -0xa943d(%rip),%rcx        # 20f6e <anon.ca35c66e77da7ac5f12473bd4dd79fb8.4.llvm.7343412667217649661+0x3d7>
   ca3ab:	mov    %rcx,0x10(%rax)
   ca3af:	movq   $0x5,0x18(%rax)
   ca3b7:	movq   $0x3,(%rax)
   ca3be:	ret
   ca3bf:	movq   $0x9,0x8(%rax)
   ca3c7:	lea    -0xb326e(%rip),%rcx        # 17160 <anon.f8fa127e3698a51bcdecbcd58237901a.58.llvm.13641424853953997931+0x40>
   ca3ce:	mov    %rcx,0x10(%rax)
   ca3d2:	movq   $0x10,0x18(%rax)
   ca3da:	movq   $0x3,(%rax)
   ca3e1:	ret
   ca3e2:	movq   $0x9,0x8(%rax)
   ca3ea:	lea    -0xa9488(%rip),%rcx        # 20f69 <anon.ca35c66e77da7ac5f12473bd4dd79fb8.4.llvm.7343412667217649661+0x3d2>
   ca3f1:	mov    %rcx,0x10(%rax)
   ca3f5:	movq   $0x5,0x18(%rax)
   ca3fd:	movq   $0x3,(%rax)
   ca404:	ret
   ca405:	movq   $0x9,0x8(%rax)
   ca40d:	lea    -0xa94bf(%rip),%rcx        # 20f55 <anon.ca35c66e77da7ac5f12473bd4dd79fb8.4.llvm.7343412667217649661+0x3be>
   ca414:	mov    %rcx,0x10(%rax)
   ca418:	movq   $0x14,0x18(%rax)
   ca420:	movq   $0x3,(%rax)
   ca427:	ret
   ca428:	movq   $0x9,0x8(%rax)
   ca430:	lea    -0xa94f7(%rip),%rcx        # 20f40 <anon.ca35c66e77da7ac5f12473bd4dd79fb8.4.llvm.7343412667217649661+0x3a9>
   ca437:	mov    %rcx,0x10(%rax)
   ca43b:	movq   $0x15,0x18(%rax)
   ca443:	movq   $0x3,(%rax)
   ca44a:	ret
   ca44b:	test   $0x1,%dl
   ca44e:	je     ca459 <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x169>
   ca450:	cmpl   $0xa0d200a,0x49(%rsi)
   ca457:	jne    ca49b <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x1ab>
   ca459:	mov    $0x50,%edi
   ca45e:	cmpb   $0x0,-0x1(%rsi,%rdi,1)
   ca463:	jne    ca478 <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x188>
   ca465:	cmp    $0x200,%rdi
   ca46c:	je     ca4be <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x1ce>
   ca46e:	cmpb   $0x0,(%rsi,%rdi,1)
   ca472:	lea    0x2(%rdi),%rdi
   ca476:	je     ca45e <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x16e>
   ca478:	movq   $0x9,0x8(%rax)
   ca480:	lea    -0xa9569(%rip),%rcx        # 20f1e <anon.ca35c66e77da7ac5f12473bd4dd79fb8.4.llvm.7343412667217649661+0x387>
   ca487:	mov    %rcx,0x10(%rax)
   ca48b:	movq   $0x15,0x18(%rax)
   ca493:	movq   $0x3,(%rax)
   ca49a:	ret
   ca49b:	movq   $0xa,0x8(%rax)
   ca4a3:	lea    -0xa9577(%rip),%rcx        # 20f33 <anon.ca35c66e77da7ac5f12473bd4dd79fb8.4.llvm.7343412667217649661+0x39c>
   ca4aa:	mov    %rcx,0x10(%rax)
   ca4ae:	movq   $0xd,0x18(%rax)
   ca4b6:	movq   $0x3,(%rax)
   ca4bd:	ret
   ca4be:	cmp    $0x400,%rcx
   ca4c5:	setae  %dil
   ca4c9:	test   $0x1ff,%ecx
   ca4cf:	sete   %r10b
   ca4d3:	test   %r10b,%dil
   ca4d6:	je     ca501 <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x211>
   ca4d8:	cmp    0x8(%r8),%rcx
   ca4dc:	jbe    ca524 <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x234>
   ca4de:	movq   $0xb,0x8(%rax)
   ca4e6:	lea    -0xa95e8(%rip),%rcx        # 20f05 <anon.ca35c66e77da7ac5f12473bd4dd79fb8.4.llvm.7343412667217649661+0x36e>
   ca4ed:	mov    %rcx,0x10(%rax)
   ca4f1:	movq   $0xc,0x18(%rax)
   ca4f9:	movq   $0x3,(%rax)
   ca500:	ret
   ca501:	movq   $0xa,0x8(%rax)
   ca509:	lea    -0xa95ff(%rip),%rcx        # 20f11 <anon.ca35c66e77da7ac5f12473bd4dd79fb8.4.llvm.7343412667217649661+0x37a>
   ca510:	mov    %rcx,0x10(%rax)
   ca514:	movq   $0xd,0x18(%rax)
   ca51c:	movq   $0x3,(%rax)
   ca523:	ret
   ca524:	mov    0xc(%rsi),%r11
   ca528:	mov    %r11,%rdi
   ca52b:	shr    $0x37,%rdi
   ca52f:	jne    ca817 <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x527>
   ca535:	test   %r11,%r11
   ca538:	setne  %dil
   ca53c:	test   $0x7f,%r11b
   ca540:	sete   %r10b
   ca544:	test   %r10b,%dil
   ca547:	je     ca578 <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x288>
   ca549:	mov    %r11,%r10
   ca54c:	shl    $0x9,%r10
   ca550:	cmp    (%r8),%r10
   ca553:	jbe    ca59b <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x2ab>
   ca555:	movq   $0xb,0x8(%rax)
   ca55d:	lea    -0xa966d(%rip),%rcx        # 20ef7 <anon.ca35c66e77da7ac5f12473bd4dd79fb8.4.llvm.7343412667217649661+0x360>
   ca564:	mov    %rcx,0x10(%rax)
   ca568:	movq   $0xe,0x18(%rax)
   ca570:	movq   $0x3,(%rax)
   ca577:	ret
   ca578:	movq   $0xa,0x8(%rax)
   ca580:	lea    -0xb2e77(%rip),%rcx        # 17710 <anon.786f9ca1aa6bf68944dde0bec4ae4f7b.2.llvm.4745231054059156856+0x300>
   ca587:	mov    %rcx,0x10(%rax)
   ca58b:	movq   $0x8,0x18(%rax)
   ca593:	movq   $0x3,(%rax)
   ca59a:	ret
   ca59b:	push   %rbp
   ca59c:	push   %r15
   ca59e:	push   %r14
   ca5a0:	push   %r13
   ca5a2:	push   %r12
   ca5a4:	push   %rbx
   ca5a5:	mov    %r11,%r14
   ca5a8:	shr    $0x10,%r14
   ca5ac:	and    $0xff80,%r11d
   ca5b3:	cmp    $0x1,%r11
   ca5b7:	sbb    $0xffffffffffffffff,%r14
   ca5bb:	cmp    0x18(%r8),%r14
   ca5bf:	jbe    ca5e1 <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x2f1>
   ca5c1:	movq   $0xb,0x8(%rax)
   ca5c9:	lea    -0xa96ea(%rip),%rcx        # 20ee6 <anon.ca35c66e77da7ac5f12473bd4dd79fb8.4.llvm.7343412667217649661+0x34f>
   ca5d0:	mov    %rcx,0x10(%rax)
   ca5d4:	movq   $0x11,0x18(%rax)
   ca5dc:	jmp    ca6dc <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x3ec>
   ca5e1:	mov    %rcx,%rdi
   ca5e4:	movabs $0x7fffffffffffff,%r13
   ca5ee:	mov    0x40(%rsi),%rcx
   ca5f2:	cmp    %r13,%rcx
   ca5f5:	ja     ca8bc <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x5cc>
   ca5fb:	mov    %rcx,%r11
   ca5fe:	shl    $0x9,%r11
   ca602:	test   $0x7f,%cl
   ca605:	sete   %cl
   ca608:	lea    -0x1(%r11),%rbx
   ca60c:	cmp    %rdi,%rbx
   ca60f:	setb   %bl
   ca612:	test   %bl,%cl
   ca614:	jne    ca636 <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x346>
   ca616:	movq   $0xa,0x8(%rax)
   ca61e:	lea    -0xb2ce5(%rip),%rcx        # 17940 <anon.76985b85afb607e444f230055e37566f.24.llvm.7116865095658657516+0x30>
   ca625:	mov    %rcx,0x10(%rax)
   ca629:	movq   $0x8,0x18(%rax)
   ca631:	jmp    ca6dc <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x3ec>
   ca636:	cmp    0x10(%r8),%r11
   ca63a:	jbe    ca65c <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x36c>
   ca63c:	movq   $0xb,0x8(%rax)
   ca644:	lea    -0xa9773(%rip),%rcx        # 20ed8 <anon.ca35c66e77da7ac5f12473bd4dd79fb8.4.llvm.7343412667217649661+0x341>
   ca64b:	mov    %rcx,0x10(%rax)
   ca64f:	movq   $0xe,0x18(%rax)
   ca657:	jmp    ca6dc <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x3ec>
   ca65c:	mov    0x24(%rsi),%r12
   ca660:	cmp    %r13,%r12
   ca663:	ja     ca8bc <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x5cc>
   ca669:	mov    %r12,%rbx
   ca66c:	shl    $0x9,%rbx
   ca670:	cmp    0x30(%r8),%rbx
   ca674:	jbe    ca693 <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x3a3>
   ca676:	movq   $0xb,0x8(%rax)
   ca67e:	lea    -0xb34e5(%rip),%rcx        # 171a0 <anon.f8fa127e3698a51bcdecbcd58237901a.58.llvm.13641424853953997931+0x80>
   ca685:	mov    %rcx,0x10(%rax)
   ca689:	movq   $0x10,0x18(%rax)
   ca691:	jmp    ca6dc <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x3ec>
   ca693:	mov    0x1c(%rsi),%r8
   ca697:	cmp    %r13,%r8
   ca69a:	ja     ca957 <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x667>
   ca6a0:	mov    $0xa,%r15d
   ca6a6:	lea    -0xa966d(%rip),%rcx        # 21040 <anon.ca35c66e77da7ac5f12473bd4dd79fb8.4.llvm.7343412667217649661+0x4a9>
   ca6ad:	test   %r12,%r12
   ca6b0:	je     ca6cc <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x3dc>
   ca6b2:	test   %r8,%r8
   ca6b5:	je     ca6cc <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x3dc>
   ca6b7:	shl    $0x9,%r8
   ca6bb:	mov    %r8,%rbp
   ca6be:	add    %rbx,%rbp
   ca6c1:	jb     ca962 <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x672>
   ca6c7:	cmp    %r11,%rbp
   ca6ca:	jbe    ca6ee <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x3fe>
   ca6cc:	mov    %r15,0x8(%rax)
   ca6d0:	mov    %rcx,0x10(%rax)
   ca6d4:	movq   $0xd,0x18(%rax)
   ca6dc:	movq   $0x3,(%rax)
   ca6e3:	pop    %rbx
   ca6e4:	pop    %r12
   ca6e6:	pop    %r13
   ca6e8:	pop    %r14
   ca6ea:	pop    %r15
   ca6ec:	pop    %rbp
   ca6ed:	ret
   ca6ee:	lea    0x1fc(,%r14,4),%rcx
   ca6f6:	movabs $0x7fffffffe00,%r15
   ca700:	and    %rcx,%r15
   ca703:	mov    %r15,-0x10(%rsp)
   ca708:	mov    0x38(%rsi),%r15
   ca70c:	cmp    $0xffffffffffffffff,%r15
   ca710:	je     ca73c <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x44c>
   ca712:	mov    %r11,%rcx
   ca715:	test   %r9b,%r9b
   ca718:	je     ca7c0 <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x4d0>
   ca71e:	cmp    $0x600,%rdi
   ca725:	jae    ca7b9 <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x4c9>
   ca72b:	movq   $0xa,0x8(%rax)
   ca733:	lea    -0xa986f(%rip),%rcx        # 20ecb <anon.ca35c66e77da7ac5f12473bd4dd79fb8.4.llvm.7343412667217649661+0x334>
   ca73a:	jmp    ca6d0 <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x3e0>
   ca73c:	mov    %edx,%ecx
   ca73e:	and    $0x2,%ecx
   ca741:	shr    $1,%ecx
   ca743:	or     %cl,%r9b
   ca746:	mov    %r11,%rcx
   ca749:	or     $0x600,%rcx
   ca750:	cmp    %rdi,%rcx
   ca753:	seta   %cl
   ca756:	or     %r9b,%cl
   ca759:	je     ca77b <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x48b>
   ca75b:	movq   $0xa,0x8(%rax)
   ca763:	lea    -0xa98fa(%rip),%rcx        # 20e70 <anon.ca35c66e77da7ac5f12473bd4dd79fb8.4.llvm.7343412667217649661+0x2d9>
   ca76a:	mov    %rcx,0x10(%rax)
   ca76e:	movq   $0x18,0x18(%rax)
   ca776:	jmp    ca6dc <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x3ec>
   ca77b:	mov    $0x2,%r9d
   ca781:	mov    %r9,(%rax)
   ca784:	mov    %rsi,0x8(%rax)
   ca788:	mov    -0x10(%rsp),%rcx
   ca78d:	mov    %rcx,0x10(%rax)
   ca791:	mov    %r15,0x18(%rax)
   ca795:	mov    %rcx,0x20(%rax)
   ca799:	mov    %r10,0x28(%rax)
   ca79d:	mov    %rdi,0x30(%rax)
   ca7a1:	mov    %r8,0x38(%rax)
   ca7a5:	mov    %rbx,0x40(%rax)
   ca7a9:	mov    %r11,0x48(%rax)
   ca7ad:	mov    %rcx,0x50(%rax)
   ca7b1:	mov    %edx,0x58(%rax)
   ca7b4:	jmp    ca6e3 <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x3f3>
   ca7b9:	lea    -0x600(%rdi),%rcx
   ca7c0:	cmp    %r13,%r15
   ca7c3:	ja     ca970 <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x680>
   ca7c9:	mov    $0xa,%r12d
   ca7cf:	mov    %r12,-0x18(%rsp)
   ca7d4:	lea    -0xa979b(%rip),%r12        # 21040 <anon.ca35c66e77da7ac5f12473bd4dd79fb8.4.llvm.7343412667217649661+0x4a9>
   ca7db:	mov    %r12,-0x8(%rsp)
   ca7e0:	cmpq   $0x0,-0x10(%rsp)
   ca7e6:	je     ca804 <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x514>
   ca7e8:	test   %r15,%r15
   ca7eb:	je     ca804 <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x514>
   ca7ed:	shl    $0x9,%r15
   ca7f1:	mov    %r15,%r12
   ca7f4:	add    -0x10(%rsp),%r12
   ca7f9:	jb     ca972 <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x682>
   ca7ff:	cmp    %rcx,%r12
   ca802:	jbe    ca827 <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x537>
   ca804:	mov    -0x18(%rsp),%rcx
   ca809:	mov    %rcx,0x8(%rax)
   ca80d:	mov    -0x8(%rsp),%rcx
   ca812:	jmp    ca6d0 <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x3e0>
   ca817:	movq   $0xc,0x8(%rax)
   ca81f:	movq   $0x3,(%rax)
   ca826:	ret
   ca827:	cmp    %rbp,%r15
   ca82a:	jae    ca831 <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x541>
   ca82c:	cmp    %r12,%r8
   ca82f:	jb     ca84c <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x55c>
   ca831:	test   %r9b,%r9b
   ca834:	je     ca86c <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x57c>
   ca836:	cmp    %r11,%r15
   ca839:	jbe    ca84c <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x55c>
   ca83b:	test   $0x2,%dl
   ca83e:	jne    ca915 <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x625>
   ca844:	xor    %r9d,%r9d
   ca847:	jmp    ca781 <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x491>
   ca84c:	movq   $0xa,0x8(%rax)
   ca854:	lea    -0xa99aa(%rip),%rcx        # 20eb1 <anon.ca35c66e77da7ac5f12473bd4dd79fb8.4.llvm.7343412667217649661+0x31a>
   ca85b:	mov    %rcx,0x10(%rax)
   ca85f:	movq   $0x1a,0x18(%rax)
   ca867:	jmp    ca6dc <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x3ec>
   ca86c:	mov    %rdx,%rcx
   ca86f:	and    $0x2,%rcx
   ca873:	mov    %rcx,-0x18(%rsp)
   ca878:	je     ca8c9 <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x5d9>
   ca87a:	mov    0x30(%rsi),%rsi
   ca87e:	cmp    %r13,%rsi
   ca881:	ja     ca986 <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x696>
   ca887:	mov    $0xa,%ecx
   ca88c:	lea    -0xa9853(%rip),%r9        # 21040 <anon.ca35c66e77da7ac5f12473bd4dd79fb8.4.llvm.7343412667217649661+0x4a9>
   ca893:	test   %rsi,%rsi
   ca896:	je     ca8af <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x5bf>
   ca898:	shl    $0x9,%rsi
   ca89c:	mov    %rsi,%r13
   ca89f:	add    -0x10(%rsp),%r13
   ca8a4:	jb     ca9b6 <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x6c6>
   ca8aa:	cmp    %r11,%r13
   ca8ad:	jbe    ca929 <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x639>
   ca8af:	mov    %rcx,0x8(%rax)
   ca8b3:	mov    %r9,0x10(%rax)
   ca8b7:	jmp    ca6d4 <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x3e4>
   ca8bc:	movq   $0xc,0x8(%rax)
   ca8c4:	jmp    ca6dc <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x3ec>
   ca8c9:	xor    %r9d,%r9d
   ca8cc:	mov    %rbx,%r13
   ca8cf:	add    $0x200,%r13
   ca8d6:	je     ca990 <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x6a0>
   ca8dc:	shl    $0xb,%r14
   ca8e0:	add    -0x10(%rsp),%r14
   ca8e5:	mov    -0x18(%rsp),%rcx
   ca8ea:	shr    $1,%ecx
   ca8ec:	shl    %cl,%r14
   ca8ef:	add    %r14,%r13
   ca8f2:	jb     ca9a5 <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x6b5>
   ca8f8:	cmp    %r11,%r13
   ca8fb:	jbe    ca781 <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x491>
   ca901:	movq   $0xa,0x8(%rax)
   ca909:	lea    -0xa9a88(%rip),%rcx        # 20e88 <anon.ca35c66e77da7ac5f12473bd4dd79fb8.4.llvm.7343412667217649661+0x2f1>
   ca910:	jmp    ca76a <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x47a>
   ca915:	movq   $0xa,0x8(%rax)
   ca91d:	lea    -0xb35c4(%rip),%rcx        # 17360 <anon.d625489d584c397ac22d75864c33158f.37.llvm.5936746164385555759+0x70>
   ca924:	jmp    ca685 <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x395>
   ca929:	cmp    %r12,%rsi
   ca92c:	jae    ca933 <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x643>
   ca92e:	cmp    %r13,%r15
   ca931:	jb     ca943 <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x653>
   ca933:	mov    $0x1,%r9d
   ca939:	cmp    %rbp,%rsi
   ca93c:	jae    ca8cc <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x5dc>
   ca93e:	cmp    %r13,%r8
   ca941:	jae    ca8cc <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x5dc>
   ca943:	movq   $0xa,0x8(%rax)
   ca94b:	lea    -0xa9ab2(%rip),%rcx        # 20ea0 <anon.ca35c66e77da7ac5f12473bd4dd79fb8.4.llvm.7343412667217649661+0x309>
   ca952:	jmp    ca5d0 <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x2e0>
   ca957:	mov    $0xc,%r15d
   ca95d:	jmp    ca6cc <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x3dc>
   ca962:	mov    %rbp,%rcx
   ca965:	mov    $0xc,%r15d
   ca96b:	jmp    ca6cc <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x3dc>
   ca970:	jmp    ca977 <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x687>
   ca972:	mov    %r12,-0x8(%rsp)
   ca977:	mov    $0xc,%ecx
   ca97c:	mov    %rcx,-0x18(%rsp)
   ca981:	jmp    ca804 <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x514>
   ca986:	mov    $0xc,%ecx
   ca98b:	jmp    ca8af <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x5bf>
   ca990:	movq   $0xc,0x8(%rax)
   ca998:	movq   $0x0,0x10(%rax)
   ca9a0:	jmp    ca6dc <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x3ec>
   ca9a5:	movq   $0xc,0x8(%rax)
   ca9ad:	mov    %r13,0x10(%rax)
   ca9b1:	jmp    ca6dc <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x3ec>
   ca9b6:	mov    %r13,%r9
   ca9b9:	mov    $0xc,%ecx
   ca9be:	jmp    ca8af <rvvdk_vmdk::stream::StreamHeader::parse_inner+0x5bf>
