```lua
-- show a name for the script
descr = "my description"
```

```lua
--[[
network = {
    ssid="the network ssid",
    open=true,
]]--
function detect(network)
    -- return true or false if this script can handle this network
    return network['ssid'] == "example network" and network['open']
end
```

```lua
-- optional
function connect(network)
    -- TODO: configure how to connect to the network
    wifi_set_psk('hunter2')
end
```

```lua
-- optional
function detect_portal()
    -- overwrite the captive portal test
    -- TODO: this should return all the infos needed by decap(...)
end
```

```lua
--[[
network = {
    gateway="192.0.1.1",
    dns={"192.0.1.2", "192.0.1.3"},
    ssid="example network",
    network="192.0.0.0",

    -- can be nil
    redirect="http://example.com/portal?some=query",
}
]]--
function decap(network)
    -- solve the captive portal
    -- return nil if the captive portal system is unknown
    -- return true if the captive portal has been solved
end
```

```lua
-- optional
healthcheck_interval = 30
function healthcheck()
    -- check if the network is still working
    -- this is only needed for buggy networks
end
```
