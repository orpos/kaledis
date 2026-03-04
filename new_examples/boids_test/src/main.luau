function love.load()
    -- love.window.setMode(800, 600)
    love.graphics.setBackgroundColor(0,0,0)

    particles = {}
    segments = 12        -- how many slices in the kaleidoscope
    centerX = love.graphics.getWidth()/2
    centerY = love.graphics.getHeight()/2
end

function love.update(dt)
    -- emit particles at center
    for i=1, 4 do
        table.insert(particles, {
            x = centerX,
            y = centerY,
            vx = (math.random()-0.5)*300,
            vy = (math.random()-0.5)*300,
            life = 1,
            hue = math.random()
        })
    end

    for i=#particles,1,-1 do
        local p = particles[i]
        p.life = p.life - dt
        if p.life <= 0 then
            table.remove(particles, i)
        else
            p.x = p.x + p.vx*dt
            p.y = p.y + p.vy*dt
        end
    end
end

function love.mousemoved(ax, ay, dx, dy, istouch)
    centerX= ax
    centerY = ay
end

-- basic HSL → RGB
local function hsl(h,s,l)
    local function f(n)
        local k = (n + h*6) % 6
        local a = s * math.min(l, 1-l)
        return l - a * math.max(-1, math.min(k-3, math.min(3-k,1)))
    end
    return f(5), f(3), f(1)
end

function love.draw()
    for _,p in ipairs(particles) do
        local r,g,b = hsl((p.hue + love.timer.getTime()*0.1)%1, 1, 0.5)
        love.graphics.setColor(r,g,b, p.life)

        for s=0,segments-1 do
            local angle = (math.pi*2/segments)*s
            local px = p.x - centerX
            local py = p.y - centerY
            local cosA = math.cos(angle)
            local sinA = math.sin(angle)
            local dx = px * cosA - py * sinA
            local dy = px * sinA + py * cosA
            love.graphics.circle("fill",
                centerX + dx,
                centerY + dy,
                3 + (p.life*4)
            )
        end
    end
end
