(module
  (type $t1 (func))
  (type $c1 (cont $t1))
  (func $f1)
  (table $t 1 funcref)
  (elem (table $t) (i32.const 0) funcref (ref.func $f1))
  (func (export "main") (result (ref $c1))
    (cont.new (type $c1) (ref.func $f1))
  )
)

(invoke "main")
