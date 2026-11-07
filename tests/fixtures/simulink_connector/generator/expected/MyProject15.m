if not(exist('project_path', 'var'))
	project_path = userpath;
	project_path =  fullfile(project_path, 'MyProject15');
end
if isfolder(project_path)
	proj = slproject.loadProject(project_path);
else
	proj = Simulink.createFromTemplate('Cap2SL2.sltx', 'Name', 'MyProject15', 'Folder', project_path);
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
if isfile('data/PA/PA_sub_dict_1.sldd')
	PA_sub_dict_1 = Simulink.data.dictionary.open('data/PA/PA_sub_dict_1.sldd');
else
	PA_sub_dict_1 = Simulink.data.dictionary.create('data/PA/PA_sub_dict_1.sldd');
end
importFromBaseWorkspace(PA_sub_dict_1, 'varList', {'mySubBus'}, 'existingVarsAction','overwrite');
saveChanges(PA_sub_dict_1);
clear PA_sub_dict_1;
file = proj.addFile('data/PA/PA_sub_dict_1.sldd');
addLabel(file, 'Capella Layer', 'PA');
clear elems;
elems(1) = Simulink.BusElement;
elems(1).Name = 'myElement';
elems(1).DataType = 'boolean';
myBus1 = Simulink.Bus;
myBus1.Elements = elems;
assignin('base', 'myBus1', myBus1);
clear elems;
if isfile('data/PA/PA_dict_1.sldd')
	PA_dict_1 = Simulink.data.dictionary.open('data/PA/PA_dict_1.sldd');
else
	PA_dict_1 = Simulink.data.dictionary.create('data/PA/PA_dict_1.sldd');
end
importFromBaseWorkspace(PA_dict_1, 'varList', {'myBus1'}, 'existingVarsAction','overwrite');
addDataSource(PA_dict_1, 'PA_sub_dict_1.sldd');
saveChanges(PA_dict_1);
clear PA_dict_1;
file = proj.addFile('data/PA/PA_dict_1.sldd');
addLabel(file, 'Capella Layer', 'PA');
clear elems;
elems(1) = Simulink.BusElement;
elems(1).Name = 'myElement';
elems(1).DataType = 'boolean';
myBus2 = Simulink.Bus;
myBus2.Elements = elems;
assignin('base', 'myBus2', myBus2);
clear elems;
if isfile('data/LA/LA_dict_2.sldd')
	LA_dict_2 = Simulink.data.dictionary.open('data/LA/LA_dict_2.sldd');
else
	LA_dict_2 = Simulink.data.dictionary.create('data/LA/LA_dict_2.sldd');
end
importFromBaseWorkspace(LA_dict_2, 'varList', {'myBus2'}, 'existingVarsAction','overwrite');
saveChanges(LA_dict_2);
clear LA_dict_2;
file = proj.addFile('data/LA/LA_dict_2.sldd');
addLabel(file, 'Capella Layer', 'LA');
evalin('base', "clear 'mySubBus' 'myBus2' 'myBus1'");
new_system('LA_Sys_2_LA');
set_param('LA_Sys_2_LA', 'DataDictionary', 'LA_dict_2.sldd');
add_block('simulink/Ports & Subsystems/In1', 'LA_Sys_2_LA/FIP_1');
set_param('LA_Sys_2_LA/FIP_1', 'Port', '1');
add_block('simulink/Ports & Subsystems/In1', 'LA_Sys_2_LA/FIP_2');
set_param('LA_Sys_2_LA/FIP_2', 'Port', '2');
add_block('simulink/Ports & Subsystems/In1', 'LA_Sys_2_LA/FIP_3');
set_param('LA_Sys_2_LA/FIP_3', 'Port', '3');
t_Block = add_block('simulink/Sinks/Terminator', 'LA_Sys_2_LA/Terminator', 'MakeNameUnique', 'on' );
t_handles = get_param(t_Block, 'PortHandles');
p_Block = get_param('LA_Sys_2_LA/FIP_1', 'handle');
p_handles = get_param(p_Block, 'PortHandles');
terminator = t_handles.Inport(1);
port = p_handles.Outport(1);
add_line('LA_Sys_2_LA', port, terminator);
clear t_Block t_handles p_Block p_handles terminator port
t_Block = add_block('simulink/Sinks/Terminator', 'LA_Sys_2_LA/Terminator', 'MakeNameUnique', 'on' );
t_handles = get_param(t_Block, 'PortHandles');
p_Block = get_param('LA_Sys_2_LA/FIP_2', 'handle');
p_handles = get_param(p_Block, 'PortHandles');
terminator = t_handles.Inport(1);
port = p_handles.Outport(1);
add_line('LA_Sys_2_LA', port, terminator);
clear t_Block t_handles p_Block p_handles terminator port
t_Block = add_block('simulink/Sinks/Terminator', 'LA_Sys_2_LA/Terminator', 'MakeNameUnique', 'on' );
t_handles = get_param(t_Block, 'PortHandles');
p_Block = get_param('LA_Sys_2_LA/FIP_3', 'handle');
p_handles = get_param(p_Block, 'PortHandles');
terminator = t_handles.Inport(1);
port = p_handles.Outport(1);
add_line('LA_Sys_2_LA', port, terminator);
clear t_Block t_handles p_Block p_handles terminator port
try
	Simulink.BlockDiagram.arrangeSystem('LA_Sys_2_LA');
catch
end
save_system('LA_Sys_2_LA', fullfile(project_path, 'models/LA/LA_Sys_2_LA.slx'));
file = proj.addFile('models/LA/LA_Sys_2_LA.slx');
addLabel(file, 'Capella Layer', 'LA');
new_system('PA_Sys_1_PA');
set_param('PA_Sys_1_PA', 'DataDictionary', 'PA_dict_1.sldd');
add_block('simulink/Ports & Subsystems/In1', 'PA_Sys_1_PA/FIP_1__LF_C_Sys_');
set_param('PA_Sys_1_PA/FIP_1__LF_C_Sys_', 'Port', '1');
add_block('simulink/Ports & Subsystems/Enable', 'PA_Sys_1_PA/S1_1__S1_2__S1_3');
set_param('PA_Sys_1_PA/S1_1__S1_2__S1_3', 'Description', 'S1/1, S1/2, S1/3');
add_block('CapellaElements/Subsystem/Function', 'PA_Sys_1_PA/SubSys');
CapSLBridge.internal.cleanSubsystem('PA_Sys_1_PA/SubSys');
add_block('simulink/Ports & Subsystems/In1', 'PA_Sys_1_PA/SubSys/FIP_1__LF_C_Sys_');
set_param('PA_Sys_1_PA/SubSys/FIP_1__LF_C_Sys_', 'Port', '1');
add_block('CapellaElements/ModelRef/Model_Function', 'PA_Sys_1_PA/SubSys/ref_to_Sys_2');
set_param('PA_Sys_1_PA/SubSys/ref_to_Sys_2', 'ModelFile', 'LA_Sys_2_LA')
handles = get_param('PA_Sys_1_PA/SubSys/FIP_1__LF_C_Sys_', 'PortHandles');
source = handles.Outport(1);
handles = get_param('PA_Sys_1_PA/SubSys/ref_to_Sys_2', 'PortHandles');
target = handles.Inport(2);
line = add_line('PA_Sys_1_PA/SubSys', source, target, 'autorouting','smart');
set_param(line, 'Name', 'a');
clear line handles source target;
try
	Simulink.BlockDiagram.arrangeSystem('PA_Sys_1_PA/SubSys');
catch
end
handles = get_param('PA_Sys_1_PA/FIP_1__LF_C_Sys_', 'PortHandles');
source = handles.Outport(1);
handles = get_param('PA_Sys_1_PA/SubSys', 'PortHandles');
target = handles.Inport(1);
line = add_line('PA_Sys_1_PA', source, target, 'autorouting','smart');
clear line handles source target;
try
	Simulink.BlockDiagram.arrangeSystem('PA_Sys_1_PA');
catch
end
save_system('PA_Sys_1_PA', fullfile(project_path, 'models/PA/PA_Sys_1_PA.slx'));
file = proj.addFile('models/PA/PA_Sys_1_PA.slx');
addLabel(file, 'Capella Layer', 'PA');
cd(proj.Information.ProjectStartupFolder);
clear proj file project_path;
