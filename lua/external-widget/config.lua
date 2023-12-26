local Rpc = require("external-widget.rpc")

---@class ExtWidget.WindowConfig
---@field max_width number?
---@field max_height number?
---@field x_offset number?
---@field y_offset number?

---@class ExtWidget.HoverConfig
---@field normal_font string[]?
---@field normal_font_size number?
---@field mono_font string[]?
---@field mono_font_size number?
---@field window ExtWidget.WindowConfig?

---@class ExtWidget.Config
---@field connect 'embed' | string
---@field hover ExtWidget.HoverConfig?

---@type ExtWidget.Config
local default_config = {
    connect = "embed"
}

---@param config ExtWidget.Config
local function setup(config)
    config = config or {}
    local connect = vim.F.if_nil(config.connect, "127.0.0.1:7000")
    local client
    if connect == "embed" then
        client = Rpc.Client.new_embed()
    else
        client = Rpc.Client.new_tcp(connect)
    end
    Rpc.setup_global_client(client)
    -- clear connect field
    config.connect = nil
    client:notify("update_config", config)
end

return {
    setup = setup
}
