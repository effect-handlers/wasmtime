(module
  (type $ii (func (param i32) (result i32)))

  (func $f (type $ii) (i32.mul (local.get 0) (local.get 0)))
  (func $g (type $ii) (i32.sub (i32.const 0) (local.get 0)))

  (elem declare func $f $g)

  (func (export "run") (param $x i32) (result i32)
    (call_ref (call_ref (local.get $x) (ref.func $f)) (ref.func $g))
  )

  ;;(func (export "null") (result i32)
  ;;  (call_ref (i32.const 1) (ref.null $ii))
  ;;)

)

(assert_return (invoke "run" (i32.const 0)) (i32.const 0))
(assert_return (invoke "run" (i32.const 3)) (i32.const -9))

;;(assert_trap (invoke "null") "null function")

