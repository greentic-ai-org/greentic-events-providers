(module
  (memory (export "memory") 1)
  (data (i32.const 0) "{\"plan\":{\"config_patch\":{\"webhook.path\":\"/ingest\",\"webhook.public_base_url\":\"https://example.invalid\",\"webhook.topic_prefix\":\"webhook.acme\"},\"secrets_patch\":{\"set\":{\"WEBHOOK_SIGNING_SECRET\":{\"redacted\":true,\"value\":null}},\"delete\":[]},\"webhook_ops\":[{\"op\":\"create\",\"id\":\"default\",\"url\":\"https://example.invalid/ingest\",\"metadata\":{\"provider\":\"generic\"}}],\"subscription_ops\":[],\"oauth_ops\":[],\"notes\":[\"dry-run: webhook provisioning plan\"]},\"ok\":true,\"step\":\"apply\"}")
  (func (export "run") (param i32 i32) (result i32 i32)
    (i32.const 0)
    (i32.const 465)
  )
)
