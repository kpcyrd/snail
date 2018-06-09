descr = "freifunk.net"

function detect(network)
    return network:find("freifunk.net") ~= nil
end

function decap(infos)
    -- nothing to do :)
    return true
end
