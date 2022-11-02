(module
  (type $t1 (func))
  (type $c1 (cont $t1))
  (tag $h)
  (func $f1
     (suspend $h))
  (table $t 1 funcref)
  (elem (table $t) (i32.const 0) funcref (ref.func $f1))
  (func (export "main")
      (resume (cont.new (type $c1) (ref.func $f1)))
  )
)

(invoke "main")
