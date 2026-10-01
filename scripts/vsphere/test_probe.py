import hashlib
import json
import unittest
from unittest.mock import patch, MagicMock
import probe


class ProbeTests(unittest.TestCase):
    def test_fault_message_and_unknown_detail_never_echo(self):
        raw=b'<s:Envelope xmlns:s="http://schemas.xmlsoap.org/soap/envelope/"><s:Body><s:Fault><faultstring>private-token</faultstring><detail><private-token/></detail></s:Fault></s:Body></s:Envelope>'
        with self.assertRaisesRegex(probe.ProbeError,'^SOAPFault$'):
            probe.parse_response(raw)
        with self.assertRaisesRegex(probe.ProbeError,'^InvalidLoginFault$'):
            probe.parse_response(raw.replace(b'<private-token/>',b'<InvalidLoginFault/>'))

    def test_size_dtd_and_encoding_reject(self):
        for raw in [b'x'*(probe.LIMIT+1),b'<!DOCTYPE a [<!ENTITY e "value">]><a>&e;</a>', '<!DOCTYPE a><a/>'.encode('utf-16'),b'<broken>']:
            with self.assertRaises(probe.ProbeError):probe.parse_response(raw)

    def test_pin_checked_before_any_http_or_secret(self):
        conn=MagicMock();conn.sock.getpeercert.return_value=b'certificate'
        with patch.object(probe.http.client,'HTTPSConnection',return_value=conn):
            with self.assertRaisesRegex(probe.ProbeError,'certificate_pin_mismatch'):
                probe.Client('example.invalid','0'*64).call('Login','private-password')
        conn.request.assert_not_called();conn.close.assert_called_once()

    def test_redirect_is_not_followed_and_response_is_capped(self):
        conn=MagicMock();conn.sock.getpeercert.return_value=b'certificate';response=conn.getresponse.return_value
        response.status=302
        with patch.object(probe.http.client,'HTTPSConnection',return_value=conn):
            with self.assertRaisesRegex(probe.ProbeError,'http_status_302'):
                probe.Client('example.invalid',hashlib.sha256(b'certificate').hexdigest()).call('Login','secret')
        response.read.assert_not_called();self.assertEqual(conn.request.call_count,1)
        response.status=200;response.read.return_value=b'<ok/>';response.getheaders.return_value=[]
        with patch.object(probe.http.client,'HTTPSConnection',return_value=conn):
            probe.Client('example.invalid',hashlib.sha256(b'certificate').hexdigest()).call('Logout','')
        response.read.assert_called_once_with(probe.LIMIT+1)

    def test_mutation_methods_reject_before_connect(self):
        with patch.object(probe.http.client,'HTTPSConnection') as constructor:
            for name in ['PowerOffVM_Task','ShutdownGuest','ExportVm','CreateSnapshotEx_Task']:
                with self.assertRaisesRegex(probe.ProbeError,'method_not_allowlisted'):
                    probe.Client('example.invalid','0'*64).call(name,'')
            constructor.assert_not_called()

    def test_property_missing_and_pagination_fail_closed(self):
        client=probe.Client('example.invalid','0'*64)
        for body in ['<token>private</token>','<objects><missingSet/></objects>']:
            client.call=lambda *a:probe.ET.fromstring('<response xmlns="urn:vim25">'+body+'</response>')
            with self.assertRaises(probe.ProbeError):client.properties('collector','VirtualMachine','vm-private',['runtime.powerState'])

    def test_logout_after_authenticated_discovery_error(self):
        client=probe.Client('example.invalid','0'*64);calls=[]
        service=probe.ET.fromstring('<response xmlns="urn:vim25"><returnval><sessionManager>session-private</sessionManager><propertyCollector>collector</propertyCollector><licenseManager>license</licenseManager><about/></returnval></response>')
        def call(method,body):
            calls.append(method)
            if method=='RetrieveServiceContent':return service
            return probe.ET.fromstring('<ok/>')
        client.call=call
        def fail(*a):raise probe.ProbeError('property_unavailable')
        client.properties=fail
        with self.assertRaises(probe.ProbeError):probe.inspect(client,'private-user','private-password')
        self.assertEqual(calls,['RetrieveServiceContent','Login','Logout']);self.assertIsNone(client.cookie)


if __name__=='__main__':unittest.main()
