local Utils = require("external-widget.utils")

---@class ExtWidget.Client
---@field private ch number
---@field private kind 'embed' | 'tcp'
---@field private process vim.SystemObj?
local Client = {}

local function try_port()
  local port = math.random(10000, 20000)
  local netstat =
    vim.system({ "netstat", "-atnp" }, { text = true }):wait().stdout
  if netstat == nil then
    return port
  end
  -- find is a line contains "127.0.0.1:port"
  local expect = "127.0.0.1:" .. port
  local lines = vim.split(netstat, "\n")
  for _, line in ipairs(lines) do
    if line:find(expect) then
      return try_port()
    end
  end
  return port
end

---@return ExtWidget.Client
function Client.new_embed()
  local cmd = Utils.get_package_path()
  cmd = cmd .. "/target/release/ext-widget"
  local ch = vim.fn.jobstart({ cmd, "embed" }, {
    rpc = true,
    detach = true,
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
function Client.new_tcp(_addr)
  local cmd = Utils.get_package_path()
  cmd = cmd .. "/target/release/ext-widget"
  local port = try_port()
  local addr = "127.0.0.1:" .. port

  local process = vim.system { cmd, "serve", "--addr", addr }
  local ch
  local max_try = 100
  while true do
    local succ, ret = pcall(vim.fn.sockconnect, "tcp", addr, {
      rpc = 1,
    })
    if succ then
      ch = ret
      break
    end
    vim.wait(10)
    max_try = max_try - 1
    if max_try == 0 then
      error("Failed to connect external-widget process")
    end
  end

  if ch > 0 then
    return setmetatable({
      ch = ch,
      kind = "tcp",
      process = process,
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
    self.process:kill(9)
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
