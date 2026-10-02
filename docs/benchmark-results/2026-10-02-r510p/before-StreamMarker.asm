
target/r510p/reference/stream-before:     file format elf64-x86-64


Disassembly of section .text:

00000000000c9ed0 <rvvdk_vmdk::stream::StreamMarker::parse>:
   c9ed0:	mov    %rdi,%rax
   c9ed3:	cmp    $0x10,%rdx
   c9ed7:	jne    c9f1d <rvvdk_vmdk::stream::StreamMarker::parse+0x4d>
   c9ed9:	mov    %ecx,%edx
   c9edb:	and    $0x1ff,%edx
   c9ee1:	jne    c9f1d <rvvdk_vmdk::stream::StreamMarker::parse+0x4d>
   c9ee3:	cmp    0x48(%r8),%rcx
   c9ee7:	jb     c9f1d <rvvdk_vmdk::stream::StreamMarker::parse+0x4d>
   c9ee9:	mov    (%rsi),%rdx
   c9eec:	mov    0x8(%rsi),%edi
   c9eef:	test   %rdi,%rdi
   c9ef2:	je     c9f40 <rvvdk_vmdk::stream::StreamMarker::parse+0x70>
   c9ef4:	cmp    %rdi,0x20(%r9)
   c9ef8:	jae    c9f70 <rvvdk_vmdk::stream::StreamMarker::parse+0xa0>
   c9efa:	movq   $0xb,0x8(%rax)
   c9f02:	lea    -0xa8f9d(%rip),%rcx        # 20f6c <anon.ca35c66e77da7ac5f12473bd4dd79fb8.4.llvm.7343412667217649661+0x44d>
   c9f09:	mov    %rcx,0x10(%rax)
   c9f0d:	movq   $0x16,0x18(%rax)
   c9f15:	movq   $0x5,(%rax)
   c9f1c:	ret
   c9f1d:	movq   $0xa,0x8(%rax)
   c9f25:	lea    -0xa8faa(%rip),%rcx        # 20f82 <anon.ca35c66e77da7ac5f12473bd4dd79fb8.4.llvm.7343412667217649661+0x463>
   c9f2c:	mov    %rcx,0x10(%rax)
   c9f30:	movq   $0x16,0x18(%rax)
   c9f38:	movq   $0x5,(%rax)
   c9f3f:	ret
   c9f40:	mov    0xc(%rsi),%esi
   c9f43:	cmp    $0x3,%rsi
   c9f47:	ja     ca05e <rvvdk_vmdk::stream::StreamMarker::parse+0x18e>
   c9f4d:	lea    -0xa919c(%rip),%rdi        # 20db8 <anon.ca35c66e77da7ac5f12473bd4dd79fb8.4.llvm.7343412667217649661+0x299>
   c9f54:	movslq (%rdi,%rsi,4),%r9
   c9f58:	add    %rdi,%r9
   c9f5b:	mov    %rsi,%rdi
   c9f5e:	jmp    *%r9
   c9f61:	mov    $0x4,%edi
   c9f66:	cmp    %rdi,%rdx
   c9f69:	je     c9fe1 <rvvdk_vmdk::stream::StreamMarker::parse+0x111>
   c9f6b:	jmp    ca03b <rvvdk_vmdk::stream::StreamMarker::parse+0x16b>
   c9f70:	mov    %rdx,%rsi
   c9f73:	shr    $0x37,%rsi
   c9f77:	jne    ca1d6 <rvvdk_vmdk::stream::StreamMarker::parse+0x306>
   c9f7d:	test   $0x7f,%dl
   c9f80:	jne    c9fb1 <rvvdk_vmdk::stream::StreamMarker::parse+0xe1>
   c9f82:	movabs $0x7fffffffffffff,%r9
   c9f8c:	mov    %rdx,%rsi
   c9f8f:	shl    $0x9,%rsi
   c9f93:	lea    0x10000(%rsi),%r10
   c9f9a:	lea    -0x80(%r9),%r11
   c9f9e:	cmp    %r11,%rdx
   c9fa1:	ja     ca20e <rvvdk_vmdk::stream::StreamMarker::parse+0x33e>
   c9fa7:	cmp    0x28(%r8),%r10
   c9fab:	jbe    ca0ce <rvvdk_vmdk::stream::StreamMarker::parse+0x1fe>
   c9fb1:	movq   $0xa,0x8(%rax)
   c9fb9:	lea    -0xa9067(%rip),%rcx        # 20f59 <anon.ca35c66e77da7ac5f12473bd4dd79fb8.4.llvm.7343412667217649661+0x43a>
   c9fc0:	mov    %rcx,0x10(%rax)
   c9fc4:	movq   $0x13,0x18(%rax)
   c9fcc:	movq   $0x5,(%rax)
   c9fd3:	ret
   c9fd4:	mov    0x50(%r8),%rdi
   c9fd8:	shr    $0x9,%rdi
   c9fdc:	cmp    %rdi,%rdx
   c9fdf:	jne    ca03b <rvvdk_vmdk::stream::StreamMarker::parse+0x16b>
   c9fe1:	lea    0x200(%rcx),%rdi
   c9fe8:	cmp    $0xfffffffffffffdff,%rcx
   c9fef:	ja     ca1e6 <rvvdk_vmdk::stream::StreamMarker::parse+0x316>
   c9ff5:	shl    $0x9,%rdx
   c9ff9:	mov    %rdx,%rcx
   c9ffc:	add    %rdi,%rcx
   c9fff:	jb     ca1fa <rvvdk_vmdk::stream::StreamMarker::parse+0x32a>
   ca005:	mov    0x30(%r8),%r9
   ca009:	cmp    %r9,%rcx
   ca00c:	jbe    ca081 <rvvdk_vmdk::stream::StreamMarker::parse+0x1b1>
   ca00e:	movq   $0xa,0x8(%rax)
   ca016:	lea    -0xa910b(%rip),%rcx        # 20f12 <anon.ca35c66e77da7ac5f12473bd4dd79fb8.4.llvm.7343412667217649661+0x3f3>
   ca01d:	mov    %rcx,0x10(%rax)
   ca021:	movq   $0x15,0x18(%rax)
   ca029:	movq   $0x5,(%rax)
   ca030:	ret
   ca031:	mov    $0x1,%edi
   ca036:	cmp    %rdi,%rdx
   ca039:	je     c9fe1 <rvvdk_vmdk::stream::StreamMarker::parse+0x111>
   ca03b:	movq   $0xa,0x8(%rax)
   ca043:	lea    -0xa9123(%rip),%rcx        # 20f27 <anon.ca35c66e77da7ac5f12473bd4dd79fb8.4.llvm.7343412667217649661+0x408>
   ca04a:	mov    %rcx,0x10(%rax)
   ca04e:	movq   $0x13,0x18(%rax)
   ca056:	movq   $0x5,(%rax)
   ca05d:	ret
   ca05e:	movq   $0x9,0x8(%rax)
   ca066:	lea    -0xa9133(%rip),%rcx        # 20f3a <anon.ca35c66e77da7ac5f12473bd4dd79fb8.4.llvm.7343412667217649661+0x41b>
   ca06d:	mov    %rcx,0x10(%rax)
   ca071:	movq   $0xb,0x18(%rax)
   ca079:	movq   $0x5,(%rax)
   ca080:	ret
   ca081:	lea    -0x1(%rsi),%r10d
   ca085:	cmp    $0x2,%r10d
   ca089:	jae    ca13c <rvvdk_vmdk::stream::StreamMarker::parse+0x26c>
   ca08f:	add    $0xfffffffffffffa00,%r9
   ca096:	cmp    %r9,%rcx
   ca099:	jbe    ca165 <rvvdk_vmdk::stream::StreamMarker::parse+0x295>
   ca09f:	mov    (%r8),%rcx
   ca0a2:	cmp    $0x2,%ecx
   ca0a5:	jne    ca165 <rvvdk_vmdk::stream::StreamMarker::parse+0x295>
   ca0ab:	movq   $0xa,0x8(%rax)
   ca0b3:	lea    -0xa91e2(%rip),%rcx        # 20ed8 <anon.ca35c66e77da7ac5f12473bd4dd79fb8.4.llvm.7343412667217649661+0x3b9>
   ca0ba:	mov    %rcx,0x10(%rax)
   ca0be:	movq   $0x22,0x18(%rax)
   ca0c6:	movq   $0x5,(%rax)
   ca0cd:	ret
   ca0ce:	or     $0xc,%rcx
   ca0d2:	mov    %rcx,%r10
   ca0d5:	add    %rdi,%r10
   ca0d8:	jb     ca222 <rvvdk_vmdk::stream::StreamMarker::parse+0x352>
   ca0de:	mov    %r10,%rdx
   ca0e1:	shr    $0x9,%rdx
   ca0e5:	and    $0x1ff,%r10d
   ca0ec:	cmp    $0x1,%r10
   ca0f0:	sbb    $0xffffffffffffffff,%rdx
   ca0f4:	cmp    %r9,%rdx
   ca0f7:	ja     ca23b <rvvdk_vmdk::stream::StreamMarker::parse+0x36b>
   ca0fd:	shl    $0x9,%rdx
   ca101:	mov    0x30(%r8),%r9
   ca105:	cmpl   $0x2,(%r8)
   ca109:	lea    -0x600(%r9),%r8
   ca110:	cmovne %r9,%r8
   ca114:	cmp    %r8,%rdx
   ca117:	jbe    ca14d <rvvdk_vmdk::stream::StreamMarker::parse+0x27d>
   ca119:	movq   $0xa,0x8(%rax)
   ca121:	lea    -0xa91e3(%rip),%rcx        # 20f45 <anon.ca35c66e77da7ac5f12473bd4dd79fb8.4.llvm.7343412667217649661+0x426>
   ca128:	mov    %rcx,0x10(%rax)
   ca12c:	movq   $0x14,0x18(%rax)
   ca134:	movq   $0x5,(%rax)
   ca13b:	ret
   ca13c:	test   %esi,%esi
   ca13e:	jne    ca17a <rvvdk_vmdk::stream::StreamMarker::parse+0x2aa>
   ca140:	cmp    %r9,%rdi
   ca143:	jne    ca1a3 <rvvdk_vmdk::stream::StreamMarker::parse+0x2d3>
   ca145:	movq   $0x4,(%rax)
   ca14c:	ret
   ca14d:	movq   $0x0,(%rax)
   ca154:	mov    %rsi,0x8(%rax)
   ca158:	mov    %rdx,0x10(%rax)
   ca15c:	mov    %rcx,0x18(%rax)
   ca160:	mov    %rdi,0x20(%rax)
   ca164:	ret
   ca165:	cmp    $0x2,%esi
   ca168:	jne    ca1c6 <rvvdk_vmdk::stream::StreamMarker::parse+0x2f6>
   ca16a:	movq   $0x2,(%rax)
   ca171:	mov    %rdi,0x8(%rax)
   ca175:	mov    %rdx,0x10(%rax)
   ca179:	ret
   ca17a:	lea    0x200(%rcx),%rsi
   ca181:	cmp    $0xfffffffffffffdff,%rcx
   ca188:	ja     ca252 <rvvdk_vmdk::stream::StreamMarker::parse+0x382>
   ca18e:	cmp    %r9,%rsi
   ca191:	jne    ca1a3 <rvvdk_vmdk::stream::StreamMarker::parse+0x2d3>
   ca193:	movq   $0x3,(%rax)
   ca19a:	mov    %rdi,0x8(%rax)
   ca19e:	mov    %rdx,0x10(%rax)
   ca1a2:	ret
   ca1a3:	movq   $0xa,0x8(%rax)
   ca1ab:	lea    -0xa92b8(%rip),%rcx        # 20efa <anon.ca35c66e77da7ac5f12473bd4dd79fb8.4.llvm.7343412667217649661+0x3db>
   ca1b2:	mov    %rcx,0x10(%rax)
   ca1b6:	movq   $0x18,0x18(%rax)
   ca1be:	movq   $0x5,(%rax)
   ca1c5:	ret
   ca1c6:	movq   $0x1,(%rax)
   ca1cd:	mov    %rdi,0x8(%rax)
   ca1d1:	mov    %rdx,0x10(%rax)
   ca1d5:	ret
   ca1d6:	movq   $0xc,0x8(%rax)
   ca1de:	movq   $0x5,(%rax)
   ca1e5:	ret
   ca1e6:	movq   $0xc,0x8(%rax)
   ca1ee:	mov    %rdi,0x10(%rax)
   ca1f2:	movq   $0x5,(%rax)
   ca1f9:	ret
   ca1fa:	movq   $0xc,0x8(%rax)
   ca202:	mov    %rcx,0x10(%rax)
   ca206:	movq   $0x5,(%rax)
   ca20d:	ret
   ca20e:	movq   $0xc,0x8(%rax)
   ca216:	mov    %r10,0x10(%rax)
   ca21a:	movq   $0x5,(%rax)
   ca221:	ret
   ca222:	movq   $0xc,0x8(%rax)
   ca22a:	mov    $0x10,%ecx
   ca22f:	mov    %r10,(%rax,%rcx,1)
   ca233:	movq   $0x5,(%rax)
   ca23a:	ret
   ca23b:	mov    $0xc,%r10d
   ca241:	mov    $0x8,%ecx
   ca246:	mov    %r10,(%rax,%rcx,1)
   ca24a:	movq   $0x5,(%rax)
   ca251:	ret
   ca252:	movq   $0xc,0x8(%rax)
   ca25a:	mov    %rsi,0x10(%rax)
   ca25e:	movq   $0x5,(%rax)
   ca265:	ret
