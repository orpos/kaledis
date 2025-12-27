function love.load()
    -- This will run once when the game loads
    print(Socket.dns.gethostname())
    print(Enet.host_create("localhost:1234", 1, 2, 2, 2))
end

function love.update(dt)
    -- This will run every frame
end

function love.draw()
    -- This will run every time a new frame will be drawn
end
