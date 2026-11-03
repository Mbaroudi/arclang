if not(exist('project_path', 'var'))
	project_path = userpath;
	project_path =  fullfile(project_path, 'MyProject18');
end
if isfolder(project_path)
	proj = slproject.loadProject(project_path);
else
	proj = Simulink.createFromTemplate('Cap2SL2.sltx', 'Name', 'MyProject18', 'Folder', project_path);
end
cd(project_path);
clear elems;
elems(1) = Simulink.BusElement;
elems(1).Name = 'myElement';
elems(1).DataType = 'boolean';
myBus1 = Simulink.Bus;
myBus1.Elements = elems;
assignin('base', 'myBus1', myBus1);
clear elems;
MyEnum = Simulink.data.dictionary.EnumTypeDefinition;
appendEnumeral(MyEnum, 'MyLiteral_1', 0, '');
appendEnumeral(MyEnum, 'MyLiteral_2', 1, '');
appendEnumeral(MyEnum, 'MyLiteral_3', 2, '');
removeEnumeral(MyEnum,1);
assignin('base', 'MyEnum', MyEnum);
if isfile('data/dict_1.sldd')
	dict_1 = Simulink.data.dictionary.open('data/dict_1.sldd');
else
	dict_1 = Simulink.data.dictionary.create('data/dict_1.sldd');
end
importFromBaseWorkspace(dict_1, 'varList', {'myBus1', 'MyEnum'}, 'existingVarsAction','overwrite');
saveChanges(dict_1);
clear dict_1;
file = proj.addFile('data/dict_1.sldd');
evalin('base', "clear 'myBus1' 'MyEnum'");
new_system('Sys_1');
set_param('Sys_1', 'DataDictionary', 'dict_1.sldd');
add_block('simulink/Ports & Subsystems/In1', 'Sys_1/FIP_1__LF_C_Sys_');
set_param('Sys_1/FIP_1__LF_C_Sys_', 'Port', '1');
add_block('simulink/Ports & Subsystems/Enable', 'Sys_1/S1_1__S1_2__S1_3');
set_param('Sys_1/S1_1__S1_2__S1_3', 'Description', 'S1/1, S1/2, S1/3');
t_Block = add_block('simulink/Sinks/Terminator', 'Sys_1/Terminator', 'MakeNameUnique', 'on' );
t_handles = get_param(t_Block, 'PortHandles');
p_Block = get_param('Sys_1/FIP_1__LF_C_Sys_', 'handle');
p_handles = get_param(p_Block, 'PortHandles');
terminator = t_handles.Inport(1);
port = p_handles.Outport(1);
add_line('Sys_1', port, terminator);
clear t_Block t_handles p_Block p_handles terminator port
try
	Simulink.BlockDiagram.arrangeSystem('Sys_1');
catch
end
save_system('Sys_1', fullfile(project_path, 'models/Sys_1.slx'));
file = proj.addFile('models/Sys_1.slx');
cd(proj.Information.ProjectStartupFolder);
clear proj file project_path;
