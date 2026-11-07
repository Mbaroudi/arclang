if not(exist('project_path', 'var'))
	project_path = userpath;
	project_path =  fullfile(project_path, 'MyProject14');
end
if isfolder(project_path)
	proj = slproject.loadProject(project_path);
else
	proj = Simulink.createFromTemplate('Cap2SL2.sltx', 'Name', 'MyProject14', 'Folder', project_path);
end
cd(project_path);
clear elems;
elems(1) = Simulink.BusElement;
elems(1).Name = 'myElement';
elems(1).DataType = 'boolean';
mySubBus = Simulink.Bus;
mySubBus.Elements = elems;
assignin('base', 'mySubBus', mySubBus);
clear elems;
if isfile('data/sub_dict_1.sldd')
	sub_dict_1 = Simulink.data.dictionary.open('data/sub_dict_1.sldd');
else
	sub_dict_1 = Simulink.data.dictionary.create('data/sub_dict_1.sldd');
end
importFromBaseWorkspace(sub_dict_1, 'varList', {'mySubBus'}, 'existingVarsAction','overwrite');
saveChanges(sub_dict_1);
clear sub_dict_1;
file = proj.addFile('data/sub_dict_1.sldd');
clear elems;
elems(1) = Simulink.BusElement;
elems(1).Name = 'myElement';
elems(1).DataType = 'boolean';
myBus1 = Simulink.Bus;
myBus1.Elements = elems;
assignin('base', 'myBus1', myBus1);
clear elems;
if isfile('data/dict_1.sldd')
	dict_1 = Simulink.data.dictionary.open('data/dict_1.sldd');
else
	dict_1 = Simulink.data.dictionary.create('data/dict_1.sldd');
end
importFromBaseWorkspace(dict_1, 'varList', {'myBus1'}, 'existingVarsAction','overwrite');
addDataSource(dict_1, 'sub_dict_1.sldd');
saveChanges(dict_1);
clear dict_1;
file = proj.addFile('data/dict_1.sldd');
clear elems;
elems(1) = Simulink.BusElement;
elems(1).Name = 'myElement';
elems(1).DataType = 'boolean';
myBus2 = Simulink.Bus;
myBus2.Elements = elems;
assignin('base', 'myBus2', myBus2);
clear elems;
if isfile('data/dict_2.sldd')
	dict_2 = Simulink.data.dictionary.open('data/dict_2.sldd');
else
	dict_2 = Simulink.data.dictionary.create('data/dict_2.sldd');
end
importFromBaseWorkspace(dict_2, 'varList', {'myBus2'}, 'existingVarsAction','overwrite');
saveChanges(dict_2);
clear dict_2;
file = proj.addFile('data/dict_2.sldd');
evalin('base', "clear 'mySubBus' 'myBus2' 'myBus1'");
new_system('Sys_2');
set_param('Sys_2', 'DataDictionary', 'dict_2.sldd');
add_block('simulink/Ports & Subsystems/In1', 'Sys_2/FIP_1');
set_param('Sys_2/FIP_1', 'Port', '1');
add_block('simulink/Ports & Subsystems/In1', 'Sys_2/FIP_2');
set_param('Sys_2/FIP_2', 'Port', '2');
add_block('simulink/Ports & Subsystems/In1', 'Sys_2/FIP_3');
set_param('Sys_2/FIP_3', 'Port', '3');
t_Block = add_block('simulink/Sinks/Terminator', 'Sys_2/Terminator', 'MakeNameUnique', 'on' );
t_handles = get_param(t_Block, 'PortHandles');
p_Block = get_param('Sys_2/FIP_1', 'handle');
p_handles = get_param(p_Block, 'PortHandles');
terminator = t_handles.Inport(1);
port = p_handles.Outport(1);
add_line('Sys_2', port, terminator);
clear t_Block t_handles p_Block p_handles terminator port
t_Block = add_block('simulink/Sinks/Terminator', 'Sys_2/Terminator', 'MakeNameUnique', 'on' );
t_handles = get_param(t_Block, 'PortHandles');
p_Block = get_param('Sys_2/FIP_2', 'handle');
p_handles = get_param(p_Block, 'PortHandles');
terminator = t_handles.Inport(1);
port = p_handles.Outport(1);
add_line('Sys_2', port, terminator);
clear t_Block t_handles p_Block p_handles terminator port
t_Block = add_block('simulink/Sinks/Terminator', 'Sys_2/Terminator', 'MakeNameUnique', 'on' );
t_handles = get_param(t_Block, 'PortHandles');
p_Block = get_param('Sys_2/FIP_3', 'handle');
p_handles = get_param(p_Block, 'PortHandles');
terminator = t_handles.Inport(1);
port = p_handles.Outport(1);
add_line('Sys_2', port, terminator);
clear t_Block t_handles p_Block p_handles terminator port
try
	Simulink.BlockDiagram.arrangeSystem('Sys_2');
catch
end
save_system('Sys_2', fullfile(project_path, 'models/Sys_2.slx'));
file = proj.addFile('models/Sys_2.slx');
new_system('Sys_1');
set_param('Sys_1', 'DataDictionary', 'dict_1.sldd');
add_block('simulink/Ports & Subsystems/In1', 'Sys_1/FIP_1__LF_C_Sys_');
set_param('Sys_1/FIP_1__LF_C_Sys_', 'Port', '1');
add_block('simulink/Ports & Subsystems/Enable', 'Sys_1/S1_1__S1_2__S1_3');
set_param('Sys_1/S1_1__S1_2__S1_3', 'Description', 'S1/1, S1/2, S1/3');
add_block('CapellaElements/Subsystem/Function', 'Sys_1/SubSys');
CapSLBridge.internal.cleanSubsystem('Sys_1/SubSys');
add_block('simulink/Ports & Subsystems/In1', 'Sys_1/SubSys/FIP_1__LF_C_Sys_');
set_param('Sys_1/SubSys/FIP_1__LF_C_Sys_', 'Port', '1');
add_block('CapellaElements/ModelRef/Model_Function', 'Sys_1/SubSys/ref_to_Sys_2');
set_param('Sys_1/SubSys/ref_to_Sys_2', 'ModelFile', 'Sys_2')
handles = get_param('Sys_1/SubSys/FIP_1__LF_C_Sys_', 'PortHandles');
source = handles.Outport(1);
handles = get_param('Sys_1/SubSys/ref_to_Sys_2', 'PortHandles');
target = handles.Inport(2);
line = add_line('Sys_1/SubSys', source, target, 'autorouting','smart');
set_param(line, 'Name', 'a');
clear line handles source target;
try
	Simulink.BlockDiagram.arrangeSystem('Sys_1/SubSys');
catch
end
handles = get_param('Sys_1/FIP_1__LF_C_Sys_', 'PortHandles');
source = handles.Outport(1);
handles = get_param('Sys_1/SubSys', 'PortHandles');
target = handles.Inport(1);
line = add_line('Sys_1', source, target, 'autorouting','smart');
clear line handles source target;
try
	Simulink.BlockDiagram.arrangeSystem('Sys_1');
catch
end
save_system('Sys_1', fullfile(project_path, 'models/Sys_1.slx'));
file = proj.addFile('models/Sys_1.slx');
cd(proj.Information.ProjectStartupFolder);
clear proj file project_path;
