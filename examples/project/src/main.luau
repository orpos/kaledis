
function love.load() 
  for i in {1,2,3,4,5}do
    print(i)
  end
  for b in {5,4,3,2,1} do
    print(b)
  end
  local unique_x = {1,2,3,4,5};
  local unique_y = {1,2,3,4,5};
    local min_width, max_width
    for x in unique_x do
        if not min_width or x < min_width then
            min_width = x
        end
        if not max_width or x > max_width then
            max_width = x
        end
    end

    local min_height, max_height
    for y in unique_y do
        if not min_height or y < min_height then
            min_height = y
        end
        if not max_height or y > max_height then
            max_height = y
        end
    end
    print(unique_y, unique_x)
end

local y = 0
local x = 0
function love.update(dt)
  if love.keyboard.isDown("down") then
    y += 5
  end
  if love.keyboard.isDown("up") then
    y -= 5
  end
  if love.keyboard.isDown("right") then
    x += 5
  end
  if love.keyboard.isDown("left") then
    x -= 5
  end
end

function love.draw()
  love.graphics.rectangle("fill",x,y,50,50)
  
end