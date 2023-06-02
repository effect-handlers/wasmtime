(module
  (type $g_type (func (result i32)))
  (type $ct (cont $g_type))
  (type $resumption_ft (func (result i32)))
  (type $resumption_ct (cont $resumption_ft))
  (tag $e (param i32))

  (func $g (result i32)
    (suspend $e (i32.const 42))
    (i32.const 666))
  (elem declare func $g)

  (func $f (export "f") (result i32)
    ;;(local $k (ref null $ct))
    (block $on_e (result i32 (ref $resumption_ct))
      (resume $ct (tag $e $on_e) (cont.new $ct (ref.func $g)))
      (unreachable))
    ;; on_e
    (drop)
    )
)

(assert_return (invoke "f") (i32.const 42))
