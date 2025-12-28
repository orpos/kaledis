local byteparse= require("@packages/byteparse")

function love.load()
    -- This will run once when the game loads
    print(byteparse.zterm_string("test"))
end

function love.update(dt)
    -- This will run every frame
end

function love.draw()
    -- This will run every time a new frame will be drawn
end