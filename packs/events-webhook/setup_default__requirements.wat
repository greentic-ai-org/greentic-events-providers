(module
  (memory (export "memory") 1)
  (data (i32.const 0) "{\"requirements\":{\"config_keys\":[\"webhook.path\",\"webhook.topic_prefix\"],\"secret_keys\":[\"WEBHOOK_SIGNING_SECRET\"],\"webhook_required\":true,\"subscriptions_required\":false}}")
  (func (export "run") (param i32 i32) (result i32 i32)
    (i32.const 0)
    (i32.const 168)
  )
)
