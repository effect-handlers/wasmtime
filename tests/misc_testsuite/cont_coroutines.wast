;; Code for running example: natural numbers via a pair of coroutines
;; Works with github.com/effect-handlers/wasm-spec#57c59550fe78f4e83b6f5c032bf1446220b41c13

;; interface for running two coroutines
;; interleaving implementation
(module $co2
  ;; type alias task = [] -> []
  (type $task (func))
  ;; type alias   ct = $task
  (type $ct   (cont $task))

  ;; yield : [] -> []
  (tag $yield (param) (result))

  ;; yield : [] -> []
  (func $yield (export "yield") (param) (result)
    (suspend $yield))

  ;; run : [(ref $task) (ref $task)] -> []
  ;; implements a 'seesaw' (c.f. Ganz et al. (ICFP@99))
  (func $run (export "run") (param $task1 (ref $task)) (param $task2 (ref $task))
    ;; locals to manage continuations
    (local $up   (ref null $ct))
    (local $down (ref null $ct))
    (local $isOtherDone i32)
    ;; initialise locals
    (local.set $up   (cont.new (type $ct) (local.get $task1)))
    (local.set $down (cont.new (type $ct) (local.get $task2)))
    ;; run $up
    (loop $h
      (block $on_yield (result (ref $ct))
        (resume (tag $yield $on_yield)
                (local.get $up))
        ;; $up finished, check whether $down is done
        (if (i32.eq (local.get $isOtherDone) (i32.const 1))
          (then (return)))
        ;; prepare to run $down
        (local.get $down)
        (local.set $up)
        (local.set $isOtherDone (i32.const 1))
        (br $h)
      ) ;; on_yield clause, stack type: [(cont $ct)]
      (local.set $up)
      (if (i32.eqz (local.get $isOtherDone))
        (then
        ;; swap $up and $down
        (local.get $down)
        (local.set $down (local.get $up))
        (local.set $up)
      ))
      (br $h)))
)
(register "co2")

;; main example: streams of odd and even naturals
(module $example
  ;; imports print : [i32] -> []
  (func $print (import "spectest" "print_i32") (param i32) (result))

  ;; imports yield : [] -> []
  (func $yield (import "co2" "yield"))

  ;; odd : [i32] -> []
  ;; prints the first $niter odd natural numbers
  (func $odd (param $niter i32)
        (local $n i32) ;; next odd number
        (local $i i32) ;; iterator
        ;; initialise locals
        (local.set $n (i32.const 1))
        (local.set $i (i32.const 1))
        (block $b
         (loop $l
          (br_if $b (i32.gt_u (local.get $i) (local.get $niter)))
          ;; print the current odd number
          (call $print (local.get $n))
          ;; compute next odd number
          (local.set $n (i32.add (local.get $n) (i32.const 2)))
          ;; increment the iterator
          (local.set $i (i32.add (local.get $i) (i32.const 1)))
          ;; yield control
          (call $yield)
          (br $l))))

  ;; even : [i32] -> []
  ;; prints the first $niter even natural numbers
  (func $even (param $niter i32)
        (local $n i32) ;; next even number
        (local $i i32) ;; iterator
        ;; initialise locals
        (local.set $n (i32.const 2))
        (local.set $i (i32.const 1))
        (block $b
         (loop $l
          (br_if $b (i32.gt_u (local.get $i) (local.get $niter)))
          (call $print (local.get $n))
          ;; compute next even number
          (local.set $n (i32.add (local.get $n) (i32.const 2)))
          ;; increment the iterator
          (local.set $i (i32.add (local.get $i) (i32.const 1)))
          ;; yield control
          (call $yield)
          (br $l))))

  ;; odd5, even5 : [] -> []
  (func $odd5 (export "odd5")
        (call $odd (i32.const 5)))
  (func $even5 (export "even5")
        (call $even (i32.const 5)))
)
(register "example")

;; example runner
(module $runner
  (type $task (func))

  ;; imports co2.run : [(ref $task) (ref $task)] -> []
  (func $run (import "co2" "run") (param (ref $task) (ref $task)))

  ;; imports $example.odd5,example.even5 : [] -> []
  (func $oddTask (import "example" "odd5"))
  (func $evenTask (import "example" "even5"))
  (elem declare func $oddTask $evenTask)

  ;; main : [] -> []
  (func $main (export "main")
    (call $run (ref.func $oddTask) (ref.func $evenTask)))
)

;; run main
(invoke "main")

