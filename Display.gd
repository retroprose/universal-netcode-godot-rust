extends Node2D


# Declare member variables here. Examples:
# var a = 2
# var b = "text"

var ascii_texture = preload("res://sprites/tilemap.png")
var invader_texture = preload("res://sprites/invaders.png")
var invader_data = null
var used_count = 0

# Called when the node enters the scene tree for the first time.
func _ready():
	
	var file = File.new()
	file.open("res://sprites/invaders.json", File.READ)
	var content = file.get_as_text()
	file.close()

	invader_data = JSON.parse(content).result
	
	used_count = 0
	
	pass # Replace with function body.

# Called every frame. 'delta' is the elapsed time since the previous frame.
#func _process(delta):
#	pass


func clear_sprites():
	used_count = 0
	for n in get_children():
		n.visible = false


func create_invader_sprite(f:int, x:float, y:float, sx:float, sy:float):
	
	if used_count >= get_child_count():
		var ns = Sprite.new()
		ns.region_enabled = true
		add_child(ns)
	
	var sprite = get_child(used_count)
	used_count += 1
	
	sprite.texture = invader_texture
	
	sprite.visible = true
	
	sprite.position = Vector2( x, y )
	
	var data = invader_data[f]
	
	sprite.scale = Vector2( sx, sy )
	
	# based on integer f
	sprite.region_rect = Rect2( data.x, 512 - data.y - data.height - 1, data.width, data.height )	
	
	return sprite


func create_ascii_sprite(f:int, x:float, y:float, sx:float, sy:float):
	
	if used_count >= get_child_count():
		var ns = Sprite.new()
		ns.region_enabled = true
		add_child(ns)
	
	var sprite = get_child(used_count)
	used_count += 1
	
	sprite.texture = ascii_texture
	
	sprite.visible = true
	
	sprite.position = Vector2( x, y )
	
	sprite.scale = Vector2( sx, sy )
	
	# compute rect from f
	var fx:int = f % 16
	var fy:int = f / 16
	sprite.region_rect = Rect2( fx * 8, fy * 8, 8, 8 )	
	
	return sprite


func create_invader(f:int, x:float, y:float, sx:float, sy:float):
	var sprite = create_invader_sprite(f, x, y, sx, sy)
	return sprite


func create_ascii(f:int, x:float, y:float, sx:float, sy:float):
	var sprite = create_ascii_sprite(f, x, y, sx, sy)
	return sprite
	
	
func create_text(x:float, y:float, w:String):
	var ch:int
	for i in w.length():
		ch = w.substr(i, 1).to_ascii()[0]
		create_ascii(ch + (15 * 256), x, y, 2.0, 2.0)
		x += 16
