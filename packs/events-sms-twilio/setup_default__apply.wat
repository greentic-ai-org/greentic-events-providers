(module
  (memory (export "memory") 1)
  (data (i32.const 0) "{\"plan\":{\"config_patch\":{\"twilio.account_sid\":\"AC1234567890\",\"twilio.from_number\":\"+15551234567\",\"twilio.webhook_path\":\"/twilio\",\"twilio.webhook_url\":\"https://example.invalid/twilio\"},\"secrets_patch\":{\"set\":{\"TWILIO_AUTH_TOKEN\":{\"redacted\":true,\"value\":null}},\"delete\":[]},\"webhook_ops\":[{\"op\":\"create\",\"id\":\"twilio-webhook\",\"url\":\"https://example.invalid/twilio\",\"metadata\":{\"provider\":\"twilio\"}}],\"subscription_ops\":[],\"oauth_ops\":[],\"notes\":[\"dry-run: sms provisioning plan\"]},\"ok\":true,\"step\":\"apply\"}")
  (func (export "run") (param i32 i32) (result i32 i32)
    (i32.const 0)
    (i32.const 505)
  )
)
