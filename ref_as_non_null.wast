(module
  (type $t (func (result i32)))

  (func $nn (param $r (ref $t)) (result (ref $t))
    (ref.as_non_null (local.get $r))
  )
  (func $n (param $r funcref) (result (ref func))
    (ref.as_non_null (local.get $r))
  )

  (table $tbl 1 funcref)
  (elem (table $tbl) (offset (i32.const 0)) func $f)
  (func $f (result i32) (i32.const 7))

  (func (export "nullable-null") (result (ref func)) (call $n (ref.null $t)))
  (func (export "nullable-f")
    (i32.const 0)
    (table.get $tbl)
    (call $n)
    drop)

  (func (export "unreachable") (result (ref $t))
    (unreachable)
    (ref.as_non_null)
    (call $nn)
  )
)

(assert_trap (invoke "unreachable") "unreachable")

(assert_trap (invoke "nullable-null") "null reference")
(assert_return (invoke "nullable-f"))

(assert_invalid
  (module
    (type $t (func (result i32)))
    (func $g (param $r (ref $t)) (drop (ref.as_non_null (local.get $r))))
    (func (call $g (ref.null $t)))
  )
  "type mismatch"
)


(module
  (type $t (func))
  (func (param $r (ref $t)) (drop (ref.as_non_null (local.get $r))))
  (func (param $r (ref func)) (drop (ref.as_non_null (local.get $r))))
  (func (param $r (ref extern)) (drop (ref.as_non_null (local.get $r))))
)
