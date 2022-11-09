;; Small continuation resume test
;; expected output:
;; 1 : i32
;; 2 : i32
;; 3 : i32
(module
  (func $print (import "spectest" "print_i32") (param i32) (result))
  (type $ft (func))
  (type $ct (cont $ft))
  (tag $h)
  (func $f (export "f")
    (suspend $h)
    (call $print (i32.const 2)))
  (func (export "run")
    (call $print (i32.const 1))
    (ref.func $f)
    (cont.new (type $ct))
    (resume)
    (call $print (i32.const 3)))
)

(invoke "run")

