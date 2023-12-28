local Rpc = require("external-widget.rpc")

local hover_group = vim.api.nvim_create_augroup("external_widget_hover", {
  clear = true,
})

local current_image_id = nil

---@param win number
---@param client ExtWidget.Client?
local function close_hover(win, client)
  return function()
    if current_image_id == nil then
      return
    end
    client = client or Rpc.get_global_client()
    local image_id = current_image_id
    current_image_id = nil

    vim.api.nvim_win_close(win, true)
    vim.o.eventignore = ""
    client:request("stop_hover", image_id)
    return true
  end
end

---@param client ExtWidget.Client?
local function scroll_down(client)
  client = client or Rpc.get_global_client()
  client:notify("scroll_down_hover", current_image_id)
end

---@param client ExtWidget.Client?
local function scroll_up(client)
  client = client or Rpc.get_global_client()
  client:notify("scroll_up_hover", current_image_id)
end

local close_shortcuts = {
  "<Esc>",
  "<C-c>",
  "q",
  "j",
  "k",
  "h",
  "l",
}
local function setup_dummy_buffer(win, buffer)
  vim.api.nvim_create_autocmd(
    { "FocusLost", "WinLeave", "WinClosed", "VimLeavePre" },
    {
      once = true,
      group = hover_group,
      buffer = buffer,
      callback = close_hover(win),
    }
  )
  vim.api.nvim_buf_set_keymap(
    buffer,
    "n",
    "<C-d>",
    "",
    { noremap = true, silent = true, callback = scroll_down }
  )
  vim.api.nvim_buf_set_keymap(
    buffer,
    "n",
    "<C-u>",
    "",
    { noremap = true, silent = true, callback = scroll_up }
  )
  for _, key in ipairs(close_shortcuts) do
    vim.api.nvim_buf_set_keymap(
      buffer,
      "n",
      key,
      "",
      { noremap = true, silent = true, callback = close_hover(win) }
    )
  end
end

---@param client ExtWidget.Client
---@param err any
---@param res lsp.Hover
local function hover_callback(client, err, res)
  if err ~= nil then
    return
  end

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
  current_image_id = client:request("start_hover", value)
  vim.api.nvim_create_autocmd(
    { "CursorMoved", "FocusLost", "WinLeave", "WinClosed", "VimLeavePre" },
    {
      once = true,
      group = hover_group,
      buffer = 0,
      callback = close_hover,
    }
  )
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
  setup_dummy_buffer = setup_dummy_buffer,
}
