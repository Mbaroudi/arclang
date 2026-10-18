#!/usr/bin/env python3
"""Coverage of a Capella model by its ArcLang import, by kind, in four buckets:
declared (the element is written with its id), carried (it lives on as part
of another declaration: a field, a list, a trace, nesting), gap (the language
has no place for it yet), tooling (Capella's own bookkeeping, not model content)."""
import re, sys, collections, xml.etree.ElementTree as ET
XSI = '{http://www.w3.org/2001/XMLSchema-instance}type'
capella, arc = sys.argv[1], sys.argv[2]
aird = sys.argv[3] if len(sys.argv) > 3 else None
text = open(arc, encoding='utf-8').read()
written = set(re.findall(r'"([^"\n]+)"', text))
tree = ET.parse(capella); root = tree.getroot()
parent = {c: p for p in root.iter() for c in p}
byid = {e.get('id'): e for e in root.iter() if e.get('id')}
names_declared = set(re.findall(r'^(?:data_type|enumeration) "([^"]+)"', text, re.M))
def kind_of(el): return (el.get(XSI) or '').split('.')[-1] if el is not None else ''
def arch_of(el):
    while el is not None:
        k = kind_of(el)
        if k.endswith('Analysis') or k.endswith('Architecture'): return k.split(':')[0]
        el = parent.get(el)
    return ''
def ref(el, key): v = el.get(key); return v.split('#')[-1] if v else None
def refs(el, key): return [v.split('#')[-1] for v in (el.get(key) or '').split()]
def owner_written(el):
    p = parent.get(el)
    while p is not None:
        if p.get('id') in written: return True
        if kind_of(p).endswith('Analysis') or kind_of(p).endswith('Architecture'): return False
        p = parent.get(p)
    return False

TOOLING = {'capellacommon:TransfoLink','capellacore:KeyValue','libraries:ModelInformation','re:CatalogElementLink','re:CatalogElement','re:RecCatalog','re:CatalogElementPkg','re:CompliancyDefinition','re:CompliancyDefinitionPkg','capellamodeller:SystemEngineering','datavalue:LiteralNumericValue','datavalue:LiteralBooleanValue','datavalue:BinaryExpression','datavalue:LiteralStringValue','datavalue:NumericReference','datavalue:EnumerationReference','datavalue:ValuePart','datavalue:ComplexValue','ctx:OperationalAnalysisRealization','la:SystemAnalysisRealization','pa:LogicalArchitectureRealization','epbs:PhysicalArchitectureRealization','epbs:EPBSArchitecture','oa:OperationalAnalysis','ctx:SystemAnalysis','la:LogicalArchitecture','pa:PhysicalArchitecture','filtering:FilteringModel','filtering:FilteringResults','filtering:FilteringResult','filtering:AssociatedFilteringCriterionSet','filtering:FilteringCriterion','filtering:FilteringCriterionPkg','libraries:LibraryReference','capellacore:EnumerationPropertyType','capellacore:EnumerationPropertyLiteral','capellacore:PropertyValuePkg','Requirements:DataTypeDefinition','Requirements:EnumerationDataTypeDefinition','Requirements:ModuleType','Requirements:RequirementType','Requirements:RelationType','Requirements:AttributeDefinition','Requirements:AttributeDefinitionEnumeration','Requirements:EnumValue','Requirements:Folder','CapellaRequirements:CapellaTypesFolder','CapellaRequirements:CapellaModule'}
def is_pkg(k): return k.endswith('Pkg') or k.endswith('Structure')

REASON = {}
GAP = {  # kind -> why the language has no place yet
 'oa:Role': 'operational roles',
 'interaction:InteractionUse': 'a scenario referencing another scenario',
 'interaction:ConstraintDuration': 'duration constraints of scenarios',
 'interaction:ArmTimerEvent': 'timers of scenarios',
 'interaction:CancelTimerEvent': 'timers of scenarios',
   'fa:ComponentExchangeAllocation': 'allocation of component exchanges to physical links',
   'capellacommon:GenericTrace': 'generic traces',
 'information:Unit': 'units (kept as a name on the type)',
}
counts = collections.defaultdict(lambda: collections.Counter())
examples = collections.defaultdict(list)
for el in root.iter():
    k = kind_of(el)
    if not k: continue
    eid = el.get('id')
    arch = arch_of(el)
    if k in TOOLING or is_pkg(k): bucket = 'tooling'
    elif (el.get('name') or '').startswith('Root ') and k in ('ctx:SystemFunction','la:LogicalFunction','pa:PhysicalFunction','oa:OperationalActivity'): bucket = 'tooling'
    elif k.endswith('PropertyValue') and kind_of(parent.get(el)) in ('capellacore:PropertyValuePkg','capellacore:PropertyValueGroup') and kind_of(parent.get(parent.get(el)) if parent.get(el) is not None else None) == 'capellacore:PropertyValuePkg': bucket = 'tooling'
    elif k == 'capellacore:PropertyValueGroup' and kind_of(parent.get(el)) == 'capellacore:PropertyValuePkg': bucket = 'tooling'
    elif eid and eid in written: bucket = 'declared'
    elif k in ('datatype:NumericType','datatype:StringType','datatype:BooleanType','datatype:Enumeration','datatype:PhysicalQuantity') and (el.get('name') or '') in names_declared: bucket = 'carried'
    elif k == 'capellacommon:StateTransition' and kind_of(byid.get(ref(el,'source'))) == 'capellacommon:InitialPseudoState': bucket = 'carried'
    elif k == 'fa:ComponentPort' and arch == 'pa': bucket = 'gap'; REASON[k] = 'ports of the root physical component (the root is the physical system itself)'
    elif k.endswith('Realization') and kind_of(parent.get(el)) in ('pa:PhysicalFunction','la:LogicalFunction','la:CapabilityRealization','fa:FunctionalChain') and (parent.get(el).get('id') not in written): bucket = 'gap'; REASON[k] = 'realizations owned by elements the language has no place for'
    elif k in GAP: bucket = 'gap'
    # carried: lives on inside another declaration
    elif k == 'cs:Part': bucket = 'carried' if ref(el, 'abstractType') in written else 'missing'
    elif k in ('capellacommon:Region', 'interaction:InstanceRole', 'interaction:MessageEnd', 'interaction:Execution', 'interaction:ExecutionEnd', 'interaction:ExecutionEvent', 'interaction:EventSentOperation', 'interaction:EventReceiptOperation', 'interaction:FragmentEnd', 'interaction:InteractionOperand', 'interaction:CombinedFragment', 'information:Property', 'information:UnionProperty', 'datavalue:EnumerationLiteral', 'information:ExchangeItemElement', 'capellacore:Constraint', 'datavalue:OpaqueExpression', 'capellacore:PropertyValueGroup', 'capellacore:StringPropertyValue', 'capellacore:IntegerPropertyValue', 'capellacore:FloatPropertyValue', 'capellacore:BooleanPropertyValue', 'capellacore:EnumerationPropertyValue', 'cs:ExchangeItemAllocation', 'cs:InterfaceUse', 'cs:InterfaceImplementation', 'capellacommon:InitialPseudoState', 'capellacommon:ChangeEvent', 'capellacommon:TimeEvent', 'Requirements:IntegerValueAttribute', 'Requirements:StringValueAttribute', 'Requirements:BooleanValueAttribute', 'Requirements:RealValueAttribute', 'Requirements:DateValueAttribute', 'Requirements:EnumerationValueAttribute', 'cs:PhysicalPathInvolvement', 'fa:FunctionalChainInvolvementFunction', 'fa:FunctionalChainInvolvementLink', 'fa:FunctionalChainReference', 'interaction:InteractionState', 'fa:ComponentFunctionalAllocation', 'deployment:PartDeploymentLink', 'fa:ExchangeCategory', 'fa:ComponentExchangeCategory', 'interaction:AbstractCapabilityExtend', 'interaction:AbstractCapabilityInclude', 'interaction:AbstractCapabilityGeneralization', 'capellacore:Generalization', 'information:PortAllocation', 'fa:ComponentPortAllocation', 'fa:ComponentExchangeFunctionalExchangeAllocation', 'ctx:CapabilityExploitation', 'ctx:MissionInvolvement', 'ctx:CapabilityInvolvement', 'oa:EntityOperationalCapabilityInvolvement', 'interaction:AbstractFunctionAbstractCapabilityInvolvement', 'interaction:FunctionalChainAbstractCapabilityInvolvement', 'capellacommon:CapabilityRealizationInvolvement', 'cybersecurity:ThreatApplication', 'cybersecurity:ThreatInvolvement', 'cybersecurity:PrimaryAssetMember', 'cybersecurity:SecurityNeeds', 'cybersecurity:FunctionStorage', 'cybersecurity:TrustBoundaryStorage', 'cybersecurity:ThreatSourceUse'):
        bucket = 'carried' if owner_written(el) else 'missing'
    elif k.endswith('Realization') or k.endswith('Relation'):
        s, t = ref(el, 'sourceElement') or ref(el, 'source'), ref(el, 'targetElement') or ref(el, 'target')
        if s is None: s = parent.get(el).get('id') if parent.get(el) is not None else None
        bucket = 'carried' if (s in written and t in written) else 'missing'
    else:
        bucket = 'missing'
    counts[k][bucket] += 1
    if bucket == 'missing' and len(examples[k]) < 3: examples[k].append(el.get('name') or eid or '?')

totals = collections.Counter()
for k, c in counts.items():
    for b, n in c.items(): totals[b] += n
semantic = totals['declared'] + totals['carried'] + totals['gap'] + totals['missing']
covered = totals['declared'] + totals['carried']
print(f"{capella.split('/')[-1]}: {semantic} model elements (+{totals['tooling']} tooling/packages ignored)")
print(f"  declared with their id: {totals['declared']}   carried inside a declaration: {totals['carried']}   => {covered} covered ({100*covered//max(semantic,1)}%)")
print(f"  language gaps: {totals['gap']} ({100*totals['gap']//max(semantic,1)}%)   not imported yet: {totals['missing']}")
def show(bucket, title):
    rows = sorted(((c[bucket], k) for k, c in counts.items() if c[bucket]), reverse=True)
    if not rows: return
    print(f"  -- {title}")
    for n, k in rows:
        extra = (GAP.get(k) or REASON.get(k, '')) if bucket == 'gap' else (', '.join(examples[k])[:60] if bucket == 'missing' else '')
        print(f"     {n:5d}  {k:55s} {extra}")
show('gap', 'language gaps (the compiler has no place for these yet)')
show('missing', 'NOT IMPORTED YET (the importer could, the language can)')
show('declared', 'declared with their id')
show('carried', 'carried inside another declaration')
if aird:
    s = open(aird, encoding='utf-8', errors='replace').read()
    names = re.findall(r'<ownedRepresentationDescriptors[^>]*?name="([^"]*)"', s, re.S)
    have = sum(1 for n in names if f'diagram "{n}"' in text or f'diagram "{n} (' in text)
    own = sum(1 for n in names if re.match(r'\s*\[(MSM|M&S|ES|FS|IS|OES|OAS|SFCD|LFCD|PFCD|OPD|PPD)\]', n))
    print(f"  diagrams: {have}/{len(names)} declared, {own} are an element's own view (scenario, machine, chain, path), {len(names)-have-own} left out")
