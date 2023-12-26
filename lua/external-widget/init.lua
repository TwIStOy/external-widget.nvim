local Config = require("external-widget.config")

---@param config ExtWidget.Config
local function setup(config)
  Config.setup(config)
end

return {
  setup = setup,
}
