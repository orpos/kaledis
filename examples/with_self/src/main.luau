local values = require("./a_package")


print(require("./a_package/c"))
print(values.test)
print(values.test2)

function love.load()
    -- This will run once when the game loads
end

function love.update(dt)
    -- This will run every frame
end

function love.draw()
    -- This will run every time a new frame will be drawn
end