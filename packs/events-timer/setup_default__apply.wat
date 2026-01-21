(module
  (memory (export "memory") 1)
  (data (i32.const 0) "{\"plan\":{\"config_patch\":{\"schedules\":[{\"name\":\"daily\",\"cron\":\"0 0 * * *\",\"topic\":\"timer.daily\"}]},\"secrets_patch\":{\"set\":{},\"delete\":[]},\"webhook_ops\":[],\"subscription_ops\":[],\"oauth_ops\":[],\"notes\":[\"dry-run: timer provisioning plan\"]},\"ok\":true,\"step\":\"apply\"}")
  (func (export "run") (param i32 i32) (result i32 i32)
    (i32.const 0)
    (i32.const 262)
  )
)
