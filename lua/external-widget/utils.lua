local M = {}

---@return string
function M.get_package_path()
    -- Path to this source file, removing the leading '@'
    local source = string.sub(debug.getinfo(1, "S").source, 2)
    -- Path to the package root
    return vim.fn.fnamemodify(source, ":p:h:h:h")
end

function M.get_term_size()
    local ffi = require('ffi')

    ffi.cdef([[
    typedef struct {
      unsigned short row;
      unsigned short col;
      unsigned short xpixel;
      unsigned short ypixel;
    } winsize;
    int ioctl(int, int, ...);
  ]]);

    local TIOCGWINSZ = null;
    if vim.fn.has("linux") == 1 then
        TIOCGWINSZ = 0x5413;
    elseif vim.fn.has("mac") == 1 then
        TIOCGWINSZ = 0x40087468
    else
        TIOCGWINSZ = 0x40087468
    end

    local sz = ffi.new('winsize')
    assert(ffi.C.ioctl(1, TIOCGWINSZ, sz) == 0, "Failed to get terminal size");
    return {
        row = sz.row,
        col = sz.col,
        xpixel = sz.xpixel,
        ypixel = sz.ypixel
    }
end

return M
