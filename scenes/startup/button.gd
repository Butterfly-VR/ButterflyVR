extends Button

@export var token_box:TextEdit

func _ready() -> void:
	@warning_ignore("unsafe_property_access", "unsafe_cast")
	(token_box.malformed_token as Signal).connect(on_invalid)

func _pressed() -> void:
	@warning_ignore("unsafe_method_access")
	token_box.parse()

func on_invalid() -> void:
	text = "INVALID TOKEN"
	add_theme_color_override("font_color", Color(0, 158, 0))
	await get_tree().create_timer(1).timeout
	text = "SUBMIT"
	remove_theme_color_override("font_color")
