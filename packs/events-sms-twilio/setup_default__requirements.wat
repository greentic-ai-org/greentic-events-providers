(module
  (memory (export "memory") 1)
  (data (i32.const 0) "{\"requirements\":{\"config_keys\":[\"twilio.account_sid\",\"twilio.from_number\",\"twilio.webhook_path\"],\"secret_keys\":[\"TWILIO_AUTH_TOKEN\"],\"webhook_required\":true,\"subscriptions_required\":false}}")
  (func (export "run") (param i32 i32) (result i32 i32)
    (i32.const 0)
    (i32.const 189)
  )
)
