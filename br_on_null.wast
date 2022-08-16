(module
  (type $t (func (result i32)))

  (func $n (param $r (ref null func)) (result i32)
    (block $l
      (br_on_null $l (local.get $r))
      (drop)
      (return (i32.const 7))
    )
    (i32.const -1)
  )

  (table $tbl 2 funcref)
  (func $f (result i32) (i32.const 5))
  (elem (table $tbl) (offset (i32.const 0)) func $f)

  (func (export "nullable-null") (result i32) (call $n (table.get $tbl (i32.const 1))))
  (func (export "nullable-non-null") (result i32) (call $n (table.get $tbl (i32.const 0))))
)

(assert_return (invoke "nullable-null") (i32.const -1))
(assert_return (invoke "nullable-non-null") (i32.const 7))

