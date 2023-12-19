local Rpc = require("external-widget.rpc")

---@param err any
---@param res lsp.Hover
local function hover_callback(err, res)
	local contents = res.contents
	if contents == nil then
		return
	end
end

local function show_hover()
	local params = require("vim.lsp.util").make_position_params()
	vim.lsp.buf_request(0, "textDocument/hover", params, hover_callback)
end

return {
	show_hover = show_hover,
}
