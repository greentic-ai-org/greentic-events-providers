(module
  (memory (export "memory") 1)
  (data (i32.const 0) "{\"subscriptions\":[{\"op\":\"create\",\"id\":\"msgraph-subscription\",\"metadata\":{\"provider\":\"msgraph\",\"resource\":\"inbox\"}},{\"op\":\"create\",\"id\":\"gmail-watch\",\"metadata\":{\"provider\":\"gmail\",\"label\":\"inbox\"}}],\"ok\":true,\"step\":\"subscriptions\"}")
  (func (export "run") (param i32 i32) (result i32 i32)
    (i32.const 0)
    (i32.const 232)
  )
)
