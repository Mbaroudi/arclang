if not(exist('project_path', 'var'))
	project_path = userpath;
	project_path =  fullfile(project_path, 'MSM_test');
end
if isfolder(project_path)
	proj = slproject.loadProject(project_path);
else
	proj = Simulink.createFromTemplate('Cap2SL2.sltx', 'Name', 'MSM_test', 'Folder', project_path);
end
cd(project_path);
if isfile('data/SA/SA_FunctionalExchanges_Categories.sldd')
	SA_FunctionalExchanges_Categories = Simulink.data.dictionary.open('data/SA/SA_FunctionalExchanges_Categories.sldd');
else
	SA_FunctionalExchanges_Categories = Simulink.data.dictionary.create('data/SA/SA_FunctionalExchanges_Categories.sldd');
end
saveChanges(SA_FunctionalExchanges_Categories);
clear SA_FunctionalExchanges_Categories;
file = proj.addFile('data/SA/SA_FunctionalExchanges_Categories.sldd');
addLabel(file, 'Capella Layer', 'SA');
if isfile('data/SA/SA_Two_regions.sldd')
	SA_Two_regions = Simulink.data.dictionary.open('data/SA/SA_Two_regions.sldd');
else
	SA_Two_regions = Simulink.data.dictionary.create('data/SA/SA_Two_regions.sldd');
end
addDataSource(SA_Two_regions, 'SA_FunctionalExchanges_Categories.sldd');
saveChanges(SA_Two_regions);
clear SA_Two_regions;
file = proj.addFile('data/SA/SA_Two_regions.sldd');
addLabel(file, 'Capella Layer', 'SA');
new_system('SA_Two_regions');
set_param('SA_Two_regions', 'Tag', 'ModelerId:SM-2');
set_param('SA_Two_regions', 'DataDictionary', 'SA_Two_regions.sldd');
states = containers.Map;
add_block('sflib/Chart', 'SA_Two_regions/Two_regions');
set_param('SA_Two_regions/Two_regions', 'Tag', 'ModelerId:SM-2');
rt = sfroot;
ch = rt.find('-isa','Stateflow.Chart','Path', 'SA_Two_regions/Two_regions');
parent = ch;
state = Stateflow.State(parent);
state.Position = [20 20 424 149];
state.Tag = 'ModelerId:RG-1';
state.labelString = 'Power';
state.Decomposition = 'EXCLUSIVE_OR';
states('RG-1') = state;
parent = states('RG-1');
state = Stateflow.State(parent);
state.Position = [142 35 90 50];
state.Tag = 'ModelerId:ST-On';
state.labelString = 'On';
state.Decomposition = 'EXCLUSIVE_OR';
states('ST-On') = state;
parent = states('RG-1');
state = Stateflow.State(parent);
state.Position = [312 35 90 50];
state.Tag = 'ModelerId:ST-Off2';
state.labelString = 'Off2';
state.Decomposition = 'EXCLUSIVE_OR';
states('ST-Off2') = state;
parent = ch;
state = Stateflow.State(parent);
state.Position = [20 179 424 100];
state.Tag = 'ModelerId:Two regions.Link';
state.labelString = 'Link';
state.Decomposition = 'EXCLUSIVE_OR';
states('Two regions.Link') = state;
parent = states('Two regions.Link');
state = Stateflow.State(parent);
state.Position = [142 194 90 50];
state.Tag = 'ModelerId:ST-Linked';
state.labelString = 'Linked';
state.Decomposition = 'EXCLUSIVE_OR';
states('ST-Linked') = state;
parent = states('Two regions.Link');
state = Stateflow.State(parent);
state.Position = [312 194 90 50];
state.Tag = 'ModelerId:ST-Lost';
state.labelString = 'Lost';
state.Decomposition = 'EXCLUSIVE_OR';
states('ST-Lost') = state;
transition = Stateflow.Transition(ch);
transition.Tag = 'ModelerId:f867c96a-7bc1-5274-9817-112ef039a22d';
transition.SourceEndPoint = [53 64];
destination = states('ST-On');
transition.Destination = destination;
transition.DestinationOClock = 8.830679737999516;
transition = Stateflow.Transition(ch);
transition.Tag = 'ModelerId:9c6725fc-bbe6-505f-853f-4116831ec5aa';
source = states('ST-On');
transition.source = source;
transition.SourceOClock = 2.955592671537678;
destination = states('ST-Off2');
transition.Destination = destination;
transition.DestinationOClock = 9.04243433400189;
transition.LabelString = [ '...Trigger:cut'];
transition = Stateflow.Transition(ch);
transition.Tag = 'ModelerId:6d896620-e021-5ed2-95fa-28d6ef33abd8';
source = states('ST-Off2');
transition.source = source;
transition.SourceOClock = 6.0829850973999715;
destination = states('ST-On');
transition.Destination = destination;
transition.DestinationOClock = 6.0829850973999715;
transition.LabelString = [ '...Trigger:restore'];
transition = Stateflow.Transition(ch);
transition.Tag = 'ModelerId:2bb0f14a-9169-569d-bdf1-97b845c98e65';
transition.SourceEndPoint = [53 223];
destination = states('ST-Linked');
transition.Destination = destination;
transition.DestinationOClock = 8.830679737999516;
transition = Stateflow.Transition(ch);
transition.Tag = 'ModelerId:44c55e65-4414-57c2-840a-75defae45b50';
source = states('ST-Linked');
transition.source = source;
transition.SourceOClock = 2.955592671537678;
destination = states('ST-Lost');
transition.Destination = destination;
transition.DestinationOClock = 9.04243433400189;
transition.LabelString = [ '...Trigger:drop'];
clear states rt ch parent transition destination junction myVar source state;
try
	Simulink.BlockDiagram.arrangeSystem('SA_Two_regions');
catch
end
save_system('SA_Two_regions', fullfile(project_path, 'models/SA/SA_Two_regions.slx'));
file = proj.addFile('models/SA/SA_Two_regions.slx');
addLabel(file, 'Capella Layer', 'SA');
cd(proj.Information.ProjectStartupFolder);
clear proj file project_path;
