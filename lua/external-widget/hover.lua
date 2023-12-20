local Rpc = require("external-widget.rpc")

local hover_group = vim.api.nvim_create_augroup("external_widget_hover", {
	clear = true,
})

---@param err any
---@param res lsp.Hover
local function hover_callback(err, res)
	local contents = res.contents
	if contents == nil then
		return
	end
	local client = Rpc.get_client()
	if client == nil then
		return
	end
	local value = contents.value

	vim.o.eventignore = "CursorHold"
	vim.api.nvim_exec_autocmds("User", {
		pattern = "ShowHover",
	})
	local image_id = vim.rpcrequest(client, "start_hover", value, contents.kind)
	vim.api.nvim_create_autocmd({
		"CursorMoved",
		"FocusLost",
		"WinLeave",
		"WinClosed",
	}, {
		once = true,
		group = hover_group,
		buffer = 0,
		callback = function()
			vim.o.eventignore = ""
			vim.rpcrequest(client, "stop_hover", image_id)
			return true
		end,
	})
end

local function show_hover()
	local params = require("vim.lsp.util").make_position_params()
	vim.lsp.buf_request(0, "textDocument/hover", params, hover_callback)
end

return {
	show_hover = show_hover,
}
