local Utils = require("external-widget.utils")

---@type number?
local ch_id = nil

local function kill_all()
	if ch_id ~= nil then
		-- vim.fn.jobstop(ch_id)
		vim.fn.chanclose(ch_id)
	end
end

local function create_client()
	if ch_id ~= nil then
		return
	end

	-- local cmd = Utils.get_package_path()
	-- cmd = cmd .. "/target/release/external-widget"
	-- local ch = vim.fn.jobstart({ cmd }, {
	-- 	rpc = 1,
	-- 	on_exit = function(_, code)
	-- 		if code ~= 0 then
	-- 			print("external-widget: failed to start server")
	-- 		end
	-- 	end,
	-- })
	local ch = vim.fn.sockconnect("tcp", "127.0.0.1:7000", {
		rpc = 1,
	})
	if ch > 0 then
		ch_id = ch
	else
		error("Failed to start external-widget process")
	end
end

local function get_client()
	if ch_id == nil then
		create_client()
	end

	return ch_id
end

vim.api.nvim_create_autocmd("VimLeavePre", {
	callback = function()
		kill_all()
	end,
})

return {
	get_client = get_client,
	kill_all = kill_all,
}
