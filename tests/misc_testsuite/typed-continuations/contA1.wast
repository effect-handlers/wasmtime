;; This file is currently a collection of many test cases
;; and will eventually be split up so that each test is in its
;; own file!

(module

  (type $unit_to_unit (func))
  (type $unit_to_int (func (result i32)))
  (type $int_to_unit (func (param i32)))
  (type $int_to_int (func (param i32) (result i32)))


  (type $f1_t (func))
  (type $f1_ct (cont $f1_t))

  (type $f2_t (func (result i32)))
  (type $f2_ct (cont $f2_t))

  (type $f3_t (func (param i32)))
  (type $f3_ct (cont $f3_t))

  (type $f4_t (func (param i32) (result i32)))
  (type $f4_ct (cont $f4_t))

  (type $f5_t (func (param i32) (result i32)))
  (type $f5_ct (cont $f5_t))

  (type $res_unit_to_unit (cont $unit_to_unit))
  (type $res_int_to_unit (cont $int_to_unit))
  (type $res_int_to_int (cont $int_to_int))
  (type $res_unit_to_int (cont $unit_to_int))

  (tag $e1_unit_to_unit)
  (tag $e2_int_to_unit (param i32))
  (tag $e3_int_to_int (param i32) (result i32))

  (tag $unused)

  (global $tracer (mut i32) (i32.const 0))
  (global $i (mut i32) (i32.const 0))



  ;; used by test cases a1 and a2
  (func $f1 (export "f1")
    (global.set  $tracer (i32.const -1))
    (suspend $e1_unit_to_unit)
    (global.set  $tracer (i32.const -2)))

  ;; used by test cases a3 and a4
  (func $f2 (export "f2") (result i32)
    (global.set  $tracer (i32.const -1))
    (suspend $e1_unit_to_unit)
    (global.set  $tracer (i32.const -2))
    (i32.const 100))

  ;; used by test cases b1 and b2
  (func $f3 (export "f3") (param $x i32)
    (global.set  $tracer (i32.const -1))
    (global.set  $i (i32.add (global.get $i) (local.get $x)))
    (suspend $e1_unit_to_unit)
    (global.set  $i (i32.add (global.get $i) (local.get $x)))
    (global.set  $tracer (i32.const -2)))


  ;; used by test cases b3 and b4
  (func $f4 (export "f4") (param $x i32) (result i32)
    (global.set  $tracer (i32.const -1))
    (global.set  $i (i32.add (global.get $i) (local.get $x)))
    (suspend $e1_unit_to_unit)
    ;;(global.set  $i (i32.add (global.get $i) (local.get $x)))
    (global.set  $tracer (i32.const -2))
    (i32.add (i32.const 2) (local.get $x)))

  (func $f5 (export "f5") (param $x i32) (result i32)
    (global.set  $tracer (i32.const -1))
    (global.set  $i (i32.add (global.get $i) (local.get $x)))
    (i32.add (i32.const 2) (local.get $x)))


  (func $test_case_a1 (export "test_case_a1") (result i32)
    (global.set  $tracer (i32.const -4))
    (block $on_e1 (result (ref $res_unit_to_unit))
      (global.set  $tracer (i32.const -8))
      (resume $f1_ct (tag $e1_unit_to_unit $on_e1) (cont.new $f1_ct (ref.func $f1)))
      (global.set  $tracer (i32.const -16))
      ;; unreachable: we never intend to invoke the resumption when handling
      ;; $e1 invoked from $f1
      (unreachable))
    ;; after on_e1, stack: [resumption]
    (global.set  $tracer (i32.const -32))
    (drop) ;; drop resumption
    (i32.const 100))

  (func $test_case_a2 (export "test_case_a2") (result i32)
    (local $finish_f1 (ref $res_unit_to_unit))
    (global.set  $tracer (i32.const -4))
    (block $on_e1 (result (ref $res_unit_to_unit))
      (global.set  $tracer (i32.const -8))
      (resume $f1_ct (tag $e1_unit_to_unit $on_e1) (cont.new $f1_ct (ref.func $f1)))
      (global.set  $tracer (i32.const -16))
      (unreachable))
    ;; after on_e1, stack: [resumption]
    (global.set  $tracer (i32.const -32))

    (resume $res_unit_to_unit)
    ;; the resume above resumes execution of f1, which finishes without further suspends
    (global.set  $tracer (i32.const -128))
    (return (i32.const 100)))

  (func $test_case_a3 (export "test_case_a3") (result i32)
    (global.set  $tracer (i32.const -4))
    (block $on_e1 (result (ref $res_unit_to_int))
      (global.set  $tracer (i32.const -8))
      (resume $f2_ct (tag $e1_unit_to_unit $on_e1) (cont.new $f2_ct (ref.func $f2)))
      (global.set  $tracer (i32.const -16))
      ;; unreachable: we never intend to invoke the resumption when handling
      ;; $e1 invoked from $f2
      (unreachable))
    ;; after on_e1, stack: [resumption]
    (global.set  $tracer (i32.const -32))
    (drop) ;; drop resumption
    (i32.const 100))

  (func $test_case_a4 (export "test_case_a4") (result i32)
    (local $finish_f2 (ref $res_unit_to_int))
    (global.set  $tracer (i32.const -4))
    (block $on_e1 (result (ref $res_unit_to_int))
      (global.set  $tracer (i32.const -8))
      (resume $f2_ct (tag $e1_unit_to_unit $on_e1) (cont.new $f2_ct (ref.func $f2)))
      (global.set  $tracer (i32.const -16))
      (unreachable))
    ;; after on_e1, stack: [resumption]
    (local.set $finish_f2)
    (global.set  $tracer (i32.const -32))
    (resume $res_unit_to_int (local.get $finish_f2))
    ;; the resume above resumes execution of f2, which finishes without further suspends
    (global.set  $tracer (i32.const -128))
    (return))

  (func $test_case_b1 (export "test_case_b1") (result i32)
    (global.set  $tracer (i32.const -4))
    (block $on_e1 (result (ref $res_unit_to_unit))
      (global.set  $tracer (i32.const -8))
      (resume $f3_ct (tag $e1_unit_to_unit $on_e1) (i32.const 100) (cont.new $f3_ct (ref.func $f3)))
      (global.set  $tracer (i32.const -16))
      ;; unreachable: we never intend to invoke the resumption when handling
      ;; $e1 invoked from $f2
      (unreachable))
    ;; after on_e1, stack: [resumption]
    (global.set  $tracer (i32.const -32))
    (drop) ;; drop resumption
    (global.get $i))

  (func $test_case_b2 (export "test_case_b2") (result i32)
    ;; remove this eventually
    (global.set  $i (i32.const 0))
    ;;(local $finish_f3 (ref $res_unit_to_unit))
    (global.set  $tracer (i32.const -4))
    (block $on_e1 (result (ref $res_unit_to_unit))
      (global.set  $tracer (i32.const -8))
      (resume $f3_ct (tag $e1_unit_to_unit $on_e1) (i32.const 50) (cont.new $f3_ct (ref.func $f3)))
      (global.set  $tracer (i32.const -16))
      (unreachable))
    ;; after on_e1, stack: [resumption]
    ;;(local.set $finish_f2)
    (global.set  $tracer (i32.const -32))
    (resume $res_unit_to_unit)
    ;; the resume above resumes execution of f2, which finishes without further suspends
    (global.set  $tracer (i32.const -128))
    (global.get $i))

  (func $test_case_b3 (export "test_case_b3") (result i32)
    ;; remove this eventually
    (global.set  $i (i32.const 0))
    (global.set  $tracer (i32.const -4))
    (block $on_e1 (result (ref $res_unit_to_int))
      (global.set  $tracer (i32.const -8))
      (resume $f4_ct (tag $e1_unit_to_unit $on_e1) (i32.const 100) (cont.new $f4_ct (ref.func $f4)))
      (global.set  $tracer (i32.const -16))
      ;; unreachable: we never intend to invoke the resumption when handling
      ;; $e1 invoked from $f2
      (unreachable))
    ;; after on_e1, stack: [resumption]
    (global.set  $tracer (i32.const -32))
    (drop) ;; drop resumption
    (global.get $i))

  (func $test_case_b4 (export "test_case_b4") (result i32)
    ;; remove this eventually
    (global.set  $i (i32.const 0))
    ;;(local $finish_f3 (ref $res_unit_to_unit))
    (global.set  $tracer (i32.const -4))
    (block $on_e1 (result (ref $res_unit_to_int))
      (global.set  $tracer (i32.const -8))
      (resume $f4_ct (tag $e1_unit_to_unit $on_e1) (i32.const 49) (cont.new $f4_ct (ref.func $f4)))
      (global.set  $tracer (i32.const -16))
      (unreachable))
    ;; after on_e1, stack: [resumption]
    ;;(local.set $finish_f2)
    (global.set  $tracer (i32.const -32))
    (resume $res_unit_to_int)
    ;; the resume above resumes execution of f2, which finishes without further suspends
    (global.set  $tracer (i32.const -128))
    (i32.add (global.get $i)))

(func $test_case_b5 (export "test_case_b5") (result i32)
    ;; remove this eventually
    (global.set  $i (i32.const 0))
    (global.set  $tracer (i32.const -4))
    (resume $f5_ct (i32.const 49) (cont.new $f5_ct (ref.func $f5)))
    (i32.add (global.get $i)))

)

(assert_return (invoke "test_case_a1") (i32.const 100))
(assert_return (invoke "test_case_a2") (i32.const 100))
(assert_return (invoke "test_case_a3") (i32.const 100))
(assert_return (invoke "test_case_a4") (i32.const 100))
;; (assert_return (invoke "test_case_b1") (i32.const 100))
;; (assert_return (invoke "test_case_b2") (i32.const 100))
;; (assert_return (invoke "test_case_b3") (i32.const 100))
;; (assert_return (invoke "test_case_b4") (i32.const 100))
;; (assert_return (invoke "test_case_b5") (i32.const 100))
