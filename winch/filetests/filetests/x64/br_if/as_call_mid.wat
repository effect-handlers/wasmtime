;;! target = "x86_64"
(module
  (func $f (param i32 i32 i32) (result i32) (i32.const -1)) 
  (func (export "as-call-mid") (result i32)
    (block (result i32)
      (call $f
        (i32.const 1) (br_if 0 (i32.const 13) (i32.const 1)) (i32.const 3)
      )
    )
  )
)
;;    0:	 55                   	push	rbp
;;    1:	 4889e5               	mov	rbp, rsp
;;    4:	 4883ec18             	sub	rsp, 0x18
;;    8:	 897c2414             	mov	dword ptr [rsp + 0x14], edi
;;    c:	 89742410             	mov	dword ptr [rsp + 0x10], esi
;;   10:	 8954240c             	mov	dword ptr [rsp + 0xc], edx
;;   14:	 4c89742404           	mov	qword ptr [rsp + 4], r14
;;   19:	 48b8ffffffff00000000 	
;; 				movabs	rax, 0xffffffff
;;   23:	 4883c418             	add	rsp, 0x18
;;   27:	 5d                   	pop	rbp
;;   28:	 c3                   	ret	
;;
;;    0:	 55                   	push	rbp
;;    1:	 4889e5               	mov	rbp, rsp
;;    4:	 4883ec08             	sub	rsp, 8
;;    8:	 4c893424             	mov	qword ptr [rsp], r14
;;    c:	 b901000000           	mov	ecx, 1
;;   11:	 48c7c00d000000       	mov	rax, 0xd
;;   18:	 85c9                 	test	ecx, ecx
;;   1a:	 0f8517000000         	jne	0x37
;;   20:	 50                   	push	rax
;;   21:	 bf01000000           	mov	edi, 1
;;   26:	 8b3424               	mov	esi, dword ptr [rsp]
;;   29:	 ba03000000           	mov	edx, 3
;;   2e:	 e800000000           	call	0x33
;;   33:	 4883c408             	add	rsp, 8
;;   37:	 4883c408             	add	rsp, 8
;;   3b:	 5d                   	pop	rbp
;;   3c:	 c3                   	ret	
