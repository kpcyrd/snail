```lua
-- show a name for the script
descr = "my description"
```

```lua
function detect(network)
    -- return true or false if this script can handle this network
    return network == "example network"
end
```

```lua
-- optional
function connect(network)
    -- TODO: configure how to connect to the network
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
function decap(infos)
    -- solve the captive portal
    -- TODO: return nil if the captive portal system is unknown?
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
