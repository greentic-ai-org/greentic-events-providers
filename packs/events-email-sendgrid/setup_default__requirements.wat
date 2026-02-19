(module
  (memory (export "memory") 1)
  (data (i32.const 0) "{\"requirements\":{\"config_keys\":[\"msgraph.client_id\",\"gmail.client_id\",\"sender.default_from\"],\"secret_keys\":[\"MSGRAPH_CLIENT_SECRET\",\"GMAIL_CLIENT_SECRET\",\"GMAIL_REFRESH_TOKEN\"],\"webhook_required\":false,\"subscriptions_required\":true}}")
  (func (export "run") (param i32 i32) (result i32 i32)
    (i32.const 0)
    (i32.const 233)
  )
)
