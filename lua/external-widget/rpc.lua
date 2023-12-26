local Utils = require("external-widget.utils")

---@class ExtWidget.Client
---@field private ch number
---@field private kind 'embed' | 'tcp'
local Client = {}

---@return ExtWidget.Client
function Client.new_embed()
	local cmd = Utils.get_package_path()
	cmd = cmd .. "/target/release/ext-widget"
	local ch = vim.fn.jobstart({ cmd, "embed" }, {
		rpc = 1,
		on_exit = function(_, code)
			if code ~= 0 then
				print("external-widget: failed to start server")
			end
		end,
	})
	if ch > 0 then
		return setmetatable({
			ch = ch,
			kind = "embed",
		}, { __index = Client })
	else
		error("Failed to start external-widget process")
	end
end

---@param addr string
---@return ExtWidget.Client
function Client.new_tcp(addr)
	local ch = vim.fn.sockconnect("tcp", addr, {
		rpc = 1,
	})
	if ch > 0 then
		return setmetatable({
			ch = ch,
			kind = "tcp",
		}, { __index = Client })
	else
		error("Failed to connect external-widget process")
	end
end

function Client:close()
	if self.kind == "embed" then
		vim.fn.jobstop(self.ch)
	else
		vim.fn.chanclose(self.ch)
	end
end

function Client:setup_autocmd()
	vim.api.nvim_create_autocmd("VimLeavePre", {
		callback = function()
			self:close()
		end,
	})
end

---@param method string
---@param ... any
---@return nil
function Client:request(method, ...)
	return vim.rpcrequest(self.ch, method, ...)
end

function Client:notify(method, ...)
	return vim.rpcnotify(self.ch, method, ...)
end

---@type ExtWidget.Client?
local GlobalClient = nil

---@param c ExtWidget.Client
local function setup_global_client(c)
	GlobalClient = c
	c:setup_autocmd()
end

return {
	Client = Client,
	setup_global_client = setup_global_client,
	get_global_client = function()
		return GlobalClient
	end,
}
