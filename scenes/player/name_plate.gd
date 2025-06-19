extends Node3D

var id:int

func _ready() -> void:
	id = get_parent().get_parent().networker.owner_id
	set_player_name("Player" + str(id))

func set_player_name(player_name:String) -> void:
	@warning_ignore("unsafe_property_access")
	get_child(0).text = player_name
