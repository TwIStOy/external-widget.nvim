local Rpc = require("external-widget.rpc")

local hover_group = vim.api.nvim_create_augroup("external_widget_hover", {
	clear = true,
})

---@param client ExtWidget.Client
---@param err any
---@param res lsp.Hover
local function hover_callback(client, err, res)
	local contents = res.contents
	if contents == nil then
		return
	end
	if client == nil then
		return
	end
	local value = contents.value

	vim.o.eventignore = "CursorHold"
	vim.api.nvim_exec_autocmds("User", {
		pattern = "ShowHover",
	})
	local image_id = client:request("start_hover", value)
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
			client:request("stop_hover", image_id)
			return true
		end,
	})
end

---@param client ExtWidget.Client?
local function show_hover(client)
	client = client or Rpc.get_global_client()
	local params = require("vim.lsp.util").make_position_params()
	vim.lsp.buf_request(0, "textDocument/hover", params, function(...)
		hover_callback(client, ...)
	end)
end

return {
	show_hover = show_hover,
}
