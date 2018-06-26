descr = "moxy"
-- NOTE: hijacks all dns traffic
-- TODO: clients3.google.com is explicitly whitelisted, so automatic decap doesn't work

function detect(network)
    return network == "#ATTHEMOXY"
end

function decap(infos)
    session = http_mksession()

    -- get redirect url
    req = http_request(session, 'GET', 'http://example.com', {})
    x = http_send(req)
    if last_err() then return end

    redirect = x['headers']['location']

    -- open captive portal page
    req = http_request(session, 'GET', redirect, {})
    x = http_send(req)
    if last_err() then return end

    -- follow https redirect
    redirect = x['headers']['location']
    req = http_request(session, 'GET', redirect, {})
    x = http_send(req)
    if last_err() then return end

    -- get fallback url
    x = html_select(x['text'], 'meta[http-equiv=\"refresh\"]')
    -- TODO: a url join function would be useful
    -- TODO: also check if there's a standard for meta tag redirects that we could implement
    redirect = x['attrs']['content']:gsub('^10; url=', 'https://enter-sh.hoisthospitality.com')

    -- open fallback form
    req = http_request(session, 'GET', redirect, {})
    x = http_send(req)
    if last_err() then return end

    -- set the reviewed checkbox
    form = {
        reviewed='on'
    }

    -- add all hidden fields to form
    hidden = html_select_list(x['text'], 'input[type="hidden"]')

    for i = 1, #hidden do
        attrs = hidden[i]['attrs']
        form[attrs['name']] = attrs['value']
    end

    -- accept the ToS
    headers = {}
    headers['Content-Type'] = 'application/x-www-form-urlencoded'

    req = http_request(session, 'POST', 'https://enter-sh.hoisthospitality.com/moxy.php/index/fallback', {
        headers=headers,
        form=form
    })
    x = http_send(req)
    if last_err() then return end

    return true
end
