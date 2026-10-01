#!/usr/bin/env python3
"""Read-only standalone ESXi discovery. Password/session stay in memory; output is allowlisted."""
import argparse
from collections import deque
import getpass
import hashlib
import hmac
import http.client
import json
import re
import ssl
import sys
import time
import xml.etree.ElementTree as ET
from xml.sax.saxutils import escape, quoteattr

NS = {'v': 'urn:vim25', 's': 'http://schemas.xmlsoap.org/soap/envelope/'}
LIMIT = 2 * 1024 * 1024
METHODS = {'RetrieveServiceContent', 'Login', 'Logout', 'RetrievePropertiesEx'}


class ProbeError(Exception):
    pass


def parse_response(raw):
    if len(raw) > LIMIT:
        raise ProbeError('response_limit_or_dtd')
    try:
        decoded = raw.decode('utf-8-sig')
    except UnicodeDecodeError:
        raise ProbeError('invalid_xml_encoding') from None
    if '<!DOCTYPE' in decoded.upper() or '<!ENTITY' in decoded.upper():
        raise ProbeError('response_limit_or_dtd')
    try:
        root = ET.fromstring(decoded)
    except ET.ParseError:
        raise ProbeError('invalid_xml') from None
    fault = root.find('.//s:Fault', NS)
    if fault is not None:
        detail = fault.find('detail')
        kind = next(iter(detail), None) if detail is not None else None
        # Never echo server fault strings, which may contain user/host identifiers.
        label = kind.tag.rsplit('}', 1)[-1] if kind is not None else 'SOAPFault'
        known = {'InvalidLoginFault', 'NotAuthenticatedFault', 'NoPermissionFault', 'InvalidPropertyFault', 'NotSupportedFault', 'RestrictedVersionFault', 'RuntimeFault'}
        raise ProbeError(label if label in known else 'SOAPFault')
    return root


class Client:
    def __init__(self, host, fingerprint):
        self.host = host
        self.fingerprint = fingerprint
        self.cookie = None
        self.timings = []
        self.context = ssl.SSLContext(ssl.PROTOCOL_TLS_CLIENT)
        self.context.check_hostname = False
        self.context.verify_mode = ssl.CERT_NONE
        self.context.minimum_version = ssl.TLSVersion.TLSv1_2

    def call(self, method, body):
        if method not in METHODS:
            raise ProbeError('method_not_allowlisted')
        start = time.monotonic()
        conn = http.client.HTTPSConnection(self.host, timeout=15, context=self.context)
        try:
            conn.connect()
            actual = hashlib.sha256(conn.sock.getpeercert(binary_form=True)).hexdigest()
            if not hmac.compare_digest(actual, self.fingerprint):
                raise ProbeError('certificate_pin_mismatch')
            # No credentials or HTTP bytes are sent before certificate verification.
            payload = ('<s:Envelope xmlns:s="http://schemas.xmlsoap.org/soap/envelope/" '
                       'xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"><s:Body>'
                       f'<{method} xmlns="urn:vim25">{body}</{method}></s:Body></s:Envelope>').encode()
            headers = {'Content-Type': 'text/xml; charset=utf-8', 'SOAPAction': '"urn:vim25/8.0"'}
            if self.cookie:
                headers['Cookie'] = self.cookie
            conn.request('POST', '/sdk', payload, headers)
            response = conn.getresponse()
            if response.status not in [200, 500]:
                raise ProbeError(f'http_status_{response.status}')
            raw = response.read(LIMIT+1)
            root = parse_response(raw)
            if response.status != 200:
                raise ProbeError('http_error_without_fault')
            for key,value in response.getheaders():
                if key.lower() == 'set-cookie' and value.startswith('vmware_soap_session='):
                    self.cookie = value.split(';', 1)[0]
            self.timings.append(dict(method=method, elapsed_ms=round((time.monotonic()-start)*1000, 3), response_bytes=len(raw)))
            return root
        finally:
            conn.close()

    def properties(self, collector, kind, reference, paths):
        body = f'<_this type="PropertyCollector">{escape(collector)}</_this><specSet><propSet><type>{escape(kind)}</type><all>false</all>'
        body += ''.join(f'<pathSet>{escape(path)}</pathSet>' for path in paths)
        body += f'</propSet><objectSet><obj type={quoteattr(kind)}>{escape(reference)}</obj><skip>false</skip></objectSet></specSet><options><maxObjects>128</maxObjects></options>'
        root = self.call('RetrievePropertiesEx', body)
        if root.find('.//v:token', NS) is not None:
            raise ProbeError('inventory_page_limit')
        if root.find('.//v:missingSet', NS) is not None:
            raise ProbeError('property_unavailable')
        return {p.findtext('v:name', namespaces=NS): p.find('v:val', NS) for p in root.findall('.//v:propSet', NS)}


def text(node, path='.'):
    if node is None:
        return None
    return node.findtext(path, namespaces=NS) if path != '.' else node.text


def refs(node):
    return [(e.get('type'), e.text) for e in node] if node is not None else []


def scalar(value):
    # Only constrained schema scalars enter the report; names/paths/keys never do.
    if value is None:
        return None
    if not re.fullmatch(r'[A-Za-z0-9_. -]{1,100}', value):
        return 'unrecognized'
    return value


def inspect(client, user, password):
    root = client.call('RetrieveServiceContent', '<_this type="ServiceInstance">ServiceInstance</_this>')
    content = root.find('.//v:returnval', NS)
    manager = text(content, 'v:sessionManager'); collector = text(content, 'v:propertyCollector')
    about = content.find('v:about', NS)
    report = {'about': {key: scalar(text(about, 'v:'+key)) for key in ['version', 'build', 'apiVersion', 'apiType', 'productLineId']},
              'vms': [], 'datastores': [], 'host': {}, 'license': {}, 'logout': False}
    try:
        client.call('Login', f'<_this type="SessionManager">{escape(manager)}</_this><userName>{escape(user)}</userName><password>{escape(password)}</password>')
        license_ref = text(content, 'v:licenseManager')
        p = client.properties(collector, 'LicenseManager', license_ref, ['licenses', 'licensedEdition'])
        report['license']['edition'] = scalar(text(p.get('licensedEdition')))
        licenses = p.get('licenses')
        report['license']['installed'] = []
        for lic in licenses if licenses is not None else []:
            fields = {text(k, 'v:key'): text(k, 'v:value') for k in lic.findall('v:properties', NS)}
            report['license']['installed'].append({'edition_key': scalar(text(lic, 'v:editionKey')),
                                                   'expiration_date': scalar(fields.get('expirationDate'))})
        pending = deque([('Folder', text(content, 'v:rootFolder'))]); seen = set()
        while pending:
            if len(pending) > 128:
                raise ProbeError('inventory_queue_limit')
            kind, reference = pending.popleft()
            if (kind, reference) in seen:
                continue
            seen.add((kind, reference))
            if len(seen) > 128:
                raise ProbeError('inventory_object_limit')
            paths = {'Folder': ['childEntity'], 'Datacenter': ['vmFolder', 'hostFolder', 'datastore'],
                     'ComputeResource': ['host', 'resourcePool'], 'ClusterComputeResource': ['host', 'resourcePool'],
                     'ResourcePool': ['vm', 'resourcePool'], 'HostSystem': ['summary.hardware', 'runtime.connectionState', 'config.product'],
                     'Datastore': ['summary', 'info'],
                     'VirtualMachine': ['runtime.powerState', 'config.guestId', 'config.hardware.device', 'config.template', 'guest.toolsRunningStatus', 'snapshot', 'disabledMethod']}.get(kind)
            if paths is None:
                continue
            p = client.properties(collector, kind, reference, paths)
            if kind == 'Folder':
                pending += refs(p.get('childEntity'))
            elif kind == 'Datacenter':
                for key in ['vmFolder', 'hostFolder']:
                    if p.get(key) is not None: pending.append((p[key].get('type'), p[key].text))
                pending += refs(p.get('datastore'))
            elif kind in ['ComputeResource', 'ClusterComputeResource', 'ResourcePool']:
                for key in ['host', 'vm', 'resourcePool']:
                    node = p.get(key)
                    if node is not None:
                        pending.extend([(node.get('type'), node.text)] if node.get('type') else refs(node))
            elif kind == 'HostSystem':
                hardware = p.get('summary.hardware')
                report['host'] = {'connection': scalar(text(p.get('runtime.connectionState'))),
                                  'cpu_threads': scalar(text(hardware, 'v:numCpuThreads')),
                                  'memory_bytes': scalar(text(hardware, 'v:memorySize'))}
            elif kind == 'Datastore':
                summary=p.get('summary'); info=p.get('info')
                report['datastores'].append({'label':f'datastore-{len(report["datastores"])+1}',
                    'type':scalar(text(summary,'v:type')), 'capacity_bytes':scalar(text(summary,'v:capacity')),
                    'free_bytes':scalar(text(summary,'v:freeSpace')), 'accessible':scalar(text(summary,'v:accessible')),
                    'vmfs_version':scalar(text(info,'v:vmfs/v:version'))})
            elif kind == 'VirtualMachine':
                disks=[]; devices=p.get('config.hardware.device')
                for device in devices if devices is not None else []:
                    if device.get('{http://www.w3.org/2001/XMLSchema-instance}type','').split(':')[-1] != 'VirtualDisk':
                        continue
                    backing=device.find('v:backing',NS)
                    disks.append({'capacity_bytes':scalar(text(device,'v:capacityInBytes')),
                                  'capacity_kib':scalar(text(device,'v:capacityInKB')),
                                  'backing_type':scalar(backing.get('{http://www.w3.org/2001/XMLSchema-instance}type','').split(':')[-1]),
                                  'disk_mode':scalar(text(backing,'v:diskMode')), 'thin':scalar(text(backing,'v:thinProvisioned')),
                                  'encrypted':backing.find('v:keyId',NS) is not None,
                                  'parent_present':backing.find('v:parent',NS) is not None})
                disabled={e.text for e in p.get('disabledMethod',[])}
                report['vms'].append({'label':f'vm-{len(report["vms"])+1}', 'guest_id':scalar(text(p.get('config.guestId'))),
                    'power_state':scalar(text(p.get('runtime.powerState'))), 'tools':scalar(text(p.get('guest.toolsRunningStatus'))),
                    'template':scalar(text(p.get('config.template'))), 'snapshot_present':p.get('snapshot') is not None,
                    'export_disabled_method_list':bool({'ExportVm','exportVm'} & disabled), 'disks':disks})
        return report
    finally:
        # Attempt logout even if Login's response was lost after server acceptance.
        if manager:
            try:
                client.call('Logout', f'<_this type="SessionManager">{escape(manager)}</_this>')
                report['logout'] = True
            except Exception:
                report['logout'] = False
            client.cookie = None


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--host',required=True)
    parser.add_argument('--certificate-sha256',required=True)
    parser.add_argument('--user',default='root')
    parser.add_argument('--repeats',type=int,default=1)
    args=parser.parse_args()
    if not re.fullmatch(r'[a-zA-Z0-9.-]+',args.host) or not re.fullmatch(r'[a-fA-F0-9]{64}',args.certificate_sha256) or not 1<=args.repeats<=3:
        parser.error('invalid host, fingerprint or repeat count')
    password=getpass.getpass('ESXi password (memory only): ')
    reports=[]
    try:
        for _ in range(args.repeats):
            client=Client(args.host,args.certificate_sha256.lower())
            report=inspect(client,args.user,password);report['requests']=client.timings
            reports.append(report)
        print(json.dumps({'schema':1,'scope':'read_only_inventory','reports':reports},indent=2))
        return 0 if all(r['logout'] for r in reports) else 1
    except ProbeError as e:
        print(json.dumps({'error':str(e)}));return 1
    except Exception as e:
        print(json.dumps({'error':type(e).__name__}));return 1
    finally:
        password=None


if __name__=='__main__':
    sys.exit(main())
