extends Node3D

@export var object:PackedScene

# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	while !(NetworkManager as NetNodeManager).id_ready():
		await get_tree().physics_frame
	if (NetworkManager as NetNodeManager).get_id() == 0:
		var instance:Node3D = object.instantiate()
		get_parent().add_child.call_deferred(instance)
		instance.position = self.position
		instance.rotation = self.rotation
	queue_free()
