;; 8 9 10

(module
  (func $print (import "spectest" "print_i32") (param i32) (result))
  (type $t1 (func))
  (type $c1 (cont $t1))
  (func $f1
    (call $print (i32.const 9)))
  (func $f2 (unreachable))
  (table $t 2 funcref)
  (elem (table $t) (i32.const 0) funcref (ref.func $f1))
  (elem (table $t) (i32.const 1) funcref (ref.func $f2))
  (tag $h)
  (func (export "main")
    (call $print (i32.const 8))
    (block $l (result (ref $c1))
      (resume (tag $h $l) (cont.new (type $c1) (ref.func $f1))) unreachable)
    (drop)
    (call $print (i32.const 10))
  )
)

(invoke "main")
(assert_trap (invoke "traps") "unreachable")
