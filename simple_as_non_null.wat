(module
  (func $n (param $r funcref) (result (ref func))
    (ref.as_non_null (local.get $r))
  )
)
