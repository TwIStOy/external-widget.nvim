local Utils = require("external-widget.utils")

local ch_id = nil
---@type vim.SystemObj | nil
local process = nil

local function start_process()
	local cmd = Utils.get_package_path()
	cmd = cmd .. "/target/release/external-widget"
	local p = vim.system({ cmd }, {}, function()
		process = nil
	end)
	process = p
end

local function kill_all()
	if process ~= nil then
		process:kill(9)
	end
end

local function create_client()
	if ch_id ~= nil then
		return
	end

	local ch = vim.fn.sockconnect("tcp", "127.0.0.1:7000")

	if ch == 0 then
		-- connect failed
		start_process()
	end

	ch = vim.fn.sockconnect("tcp", "127.0.0.1:7000")

	if ch == 0 then
		return
	end

	ch_id = ch
end

local function get_client()
	if ch_id == nil then
		create_client()
	end

	return ch_id
end


return {
	get_client = get_client,
	kill_all = kill_all,
}
