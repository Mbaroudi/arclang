if not(exist('project_path', 'var'))
	project_path = userpath;
	project_path =  fullfile(project_path, 'SimulinkScreenshot_buses_3');
end
if isfolder(project_path)
	proj = slproject.loadProject(project_path);
else
	proj = Simulink.createFromTemplate('Cap2SL2.sltx', 'Name', 'SimulinkScreenshot_buses_3', 'Folder', project_path);
end
cd(project_path);
clear elems;
elems(1) = Simulink.BusElement;
elems(1).Name = 'Value';
elems(1).DataType = 'double';
elems(2) = Simulink.BusElement;
elems(2).Name = 'Validity';
elems(2).DataType = 'boolean';
myBus = Simulink.Bus;
myBus.Elements = elems;
assignin('base', 'myBus', myBus);
clear elems;
if isfile('data/bus__1_.sldd')
	bus__1_ = Simulink.data.dictionary.open('data/bus__1_.sldd');
else
	bus__1_ = Simulink.data.dictionary.create('data/bus__1_.sldd');
end
importFromBaseWorkspace(bus__1_, 'varList', {'myBus'}, 'existingVarsAction','overwrite');
saveChanges(bus__1_);
clear bus__1_;
file = proj.addFile('data/bus__1_.sldd');
evalin('base', "clear 'myBus'");
new_system('MyProject');
set_param('MyProject', 'DataDictionary', 'bus__1_.sldd');
add_block('simulink/Ports & Subsystems/Subsystem', 'MyProject/System_A');
set_param('MyProject/System_A', 'Position', '[300 20 380 100]')
CapSLBridge.internal.cleanSubsystem('MyProject/System_A');
add_block('simulink/Ports & Subsystems/In1', 'MyProject/System_A/in1');
set_param('MyProject/System_A/in1', 'Port', '1');
set_param('MyProject/System_A/in1', 'Position', '[1 1 31 21]');
add_block('simulink/Ports & Subsystems/Subsystem', 'MyProject/System_B');
set_param('MyProject/System_B', 'Position', '[40 20 120 100]')
CapSLBridge.internal.cleanSubsystem('MyProject/System_B');
add_block('simulink/Ports & Subsystems/Out1', 'MyProject/System_B/out1');
set_param('MyProject/System_B/out1', 'Port', '1');
set_param('MyProject/System_B/out1', 'Position', '[180 30 210 50]');
set_param('MyProject/System_B/out1', 'OutDataTypeStr', 'Bus: myBus')
add_block('simulink/Sources/Constant', 'MyProject/System_B/MyValue');
set_param('MyProject/System_B/MyValue', 'Position', '[10 10 40 40]')
set_param('MyProject/System_B/MyValue', 'OutDataTypeStr', 'double')
set_param('MyProject/System_B/MyValue', 'value', '100')
handles = get_param('MyProject/System_B/MyValue', 'porthandles');
clear handles;
add_block('simulink/Sources/Constant', 'MyProject/System_B/MyValidity');
set_param('MyProject/System_B/MyValidity', 'Position', '[10 100 40 130]')
set_param('MyProject/System_B/MyValidity', 'OutDataTypeStr', 'boolean')
set_param('MyProject/System_B/MyValidity', 'value', 'true')
handles = get_param('MyProject/System_B/MyValidity', 'porthandles');
clear handles;
add_block('simulink/Signal Routing/Bus Creator', 'MyProject/System_B/creator');
set_param('MyProject/System_B/creator', 'Position', '[100 10 110 60]')
set_param('MyProject/System_B/creator', 'OutDataTypeStr', 'Bus: myBus')
handles = get_param('MyProject/System_B/creator', 'porthandles');
clear handles;
handles = get_param('MyProject/System_B/MyValue', 'PortHandles');
source = handles.Outport(1);
handles = get_param('MyProject/System_B/creator', 'PortHandles');
target = handles.Inport(1);
line = add_line('MyProject/System_B', source, target, 'autorouting','smart');
set_param(line, 'Name', 'value____creator');
clear line handles source target;
handles = get_param('MyProject/System_B/MyValidity', 'PortHandles');
source = handles.Outport(1);
handles = get_param('MyProject/System_B/creator', 'PortHandles');
target = handles.Inport(2);
line = add_line('MyProject/System_B', source, target, 'autorouting','smart');
set_param(line, 'Name', 'validity____creator');
clear line handles source target;
handles = get_param('MyProject/System_B/creator', 'PortHandles');
source = handles.Outport(1);
handles = get_param('MyProject/System_B/out1', 'PortHandles');
target = handles.Inport(1);
line = add_line('MyProject/System_B', source, target, 'autorouting','smart');
set_param(line, 'Name', 'creator____port');
clear line handles source target;
handles = get_param('MyProject/System_B', 'PortHandles');
source = handles.Outport(1);
handles = get_param('MyProject/System_A', 'PortHandles');
target = handles.Inport(1);
line = add_line('MyProject', source, target, 'autorouting','smart');
set_param(line, 'Name', 'sys_A____sys_B');
clear line handles source target;
save_system('MyProject', fullfile(project_path, 'models/MyProject.slx'));
file = proj.addFile('models/MyProject.slx');
cd(proj.Information.ProjectStartupFolder);
clear proj file project_path;
